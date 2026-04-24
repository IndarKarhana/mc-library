import Foundation
import Metal

func fail(_ message: String) -> Never {
    if let data = (message + "\n").data(using: .utf8) {
        FileHandle.standardError.write(data)
    }
    exit(1)
}

if CommandLine.arguments.count != 10 {
    fail("expected 9 arguments: source_path n_paths n_steps log_s0 strike drift_dt vol_dt discount seed")
}

let sourcePath = CommandLine.arguments[1]

guard let nPaths = Int(CommandLine.arguments[2]),
      let nSteps = Int(CommandLine.arguments[3]),
      let logS0 = Float(CommandLine.arguments[4]),
      let strike = Float(CommandLine.arguments[5]),
      let driftDt = Float(CommandLine.arguments[6]),
      let volDt = Float(CommandLine.arguments[7]),
      let discount = Float(CommandLine.arguments[8]),
      let seed = UInt32(CommandLine.arguments[9]) else {
    fail("unable to parse numeric arguments")
}

guard let device = MTLCreateSystemDefaultDevice() else {
    fail("unable to create default Metal device")
}

let source: String
do {
    source = try String(contentsOfFile: sourcePath, encoding: .utf8)
} catch {
    fail("unable to read Metal source: \(error)")
}

let library: MTLLibrary
do {
    library = try device.makeLibrary(source: source, options: nil)
} catch {
    fail("unable to compile Metal source at runtime: \(error)")
}

guard let function = library.makeFunction(name: "mc_metal_european_call_stepwise_v1") else {
    fail("unable to find kernel entry point")
}

let pipeline: MTLComputePipelineState
do {
    pipeline = try device.makeComputePipelineState(function: function)
} catch {
    fail("unable to create compute pipeline: \(error)")
}

let threadsPerGroupWidth = min(pipeline.maxTotalThreadsPerThreadgroup, 256)
let threadgroupsCount = (nPaths + threadsPerGroupWidth - 1) / threadsPerGroupWidth
let partialBufferBytes = threadgroupsCount * MemoryLayout<Float>.stride

guard let partialSumBuffer = device.makeBuffer(length: partialBufferBytes, options: .storageModeShared),
      let partialSqSumBuffer = device.makeBuffer(length: partialBufferBytes, options: .storageModeShared),
      let commandQueue = device.makeCommandQueue(),
      let commandBuffer = commandQueue.makeCommandBuffer(),
      let commandEncoder = commandBuffer.makeComputeCommandEncoder() else {
    fail("unable to create Metal buffers or command queue")
}

var nPathsI = Int32(nPaths)
var nStepsI = Int32(nSteps)
var logS0Value = logS0
var strikeValue = strike
var driftDtValue = driftDt
var volDtValue = volDt
var discountValue = discount
var seedValue = seed

commandEncoder.setComputePipelineState(pipeline)
commandEncoder.setBuffer(partialSumBuffer, offset: 0, index: 0)
commandEncoder.setBuffer(partialSqSumBuffer, offset: 0, index: 1)
commandEncoder.setBytes(&nPathsI, length: MemoryLayout<Int32>.stride, index: 2)
commandEncoder.setBytes(&nStepsI, length: MemoryLayout<Int32>.stride, index: 3)
commandEncoder.setBytes(&logS0Value, length: MemoryLayout<Float>.stride, index: 4)
commandEncoder.setBytes(&strikeValue, length: MemoryLayout<Float>.stride, index: 5)
commandEncoder.setBytes(&driftDtValue, length: MemoryLayout<Float>.stride, index: 6)
commandEncoder.setBytes(&volDtValue, length: MemoryLayout<Float>.stride, index: 7)
commandEncoder.setBytes(&discountValue, length: MemoryLayout<Float>.stride, index: 8)
commandEncoder.setBytes(&seedValue, length: MemoryLayout<UInt32>.stride, index: 9)

let threadsPerThreadgroup = MTLSize(width: threadsPerGroupWidth, height: 1, depth: 1)
let threadgroups = MTLSize(
    width: threadgroupsCount,
    height: 1,
    depth: 1
)

commandEncoder.dispatchThreadgroups(threadgroups, threadsPerThreadgroup: threadsPerThreadgroup)
commandEncoder.endEncoding()
commandBuffer.commit()
commandBuffer.waitUntilCompleted()

let partialSumsPointer = partialSumBuffer.contents().bindMemory(to: Float.self, capacity: threadgroupsCount)
let partialSqSumsPointer = partialSqSumBuffer.contents().bindMemory(to: Float.self, capacity: threadgroupsCount)
var payoffSum = 0.0
var payoffSqSum = 0.0

for groupIndex in 0..<threadgroupsCount {
    payoffSum += Double(partialSumsPointer[groupIndex])
    payoffSqSum += Double(partialSqSumsPointer[groupIndex])
}

let n = Double(nPaths)
let price = payoffSum / n
let variance = max(0.0, (payoffSqSum / n) - (price * price))
let stderr = sqrt(variance) / sqrt(n)

print("\(price),\(stderr)")
