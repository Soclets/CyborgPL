#import <Metal/Metal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main(int argc, char **argv) {
    @autoreleasepool {
    const char *mode = argv[1]; long n = atol(getenv("N"));
    id<MTLDevice> dev = MTLCreateSystemDefaultDevice();
    NSError *e = nil;
    id<MTLLibrary> lib = [dev newLibraryWithURL:[NSURL fileURLWithPath:[NSString stringWithUTF8String:getenv("METALLIB")]] error:&e];
    id<MTLComputePipelineState> ps = [dev newComputePipelineStateWithFunction:[lib newFunctionWithName:[NSString stringWithUTF8String:mode]] error:&e];
    id<MTLCommandQueue> q = [dev newCommandQueue];
    id<MTLCommandBuffer> cb = [q commandBuffer];
    id<MTLComputeCommandEncoder> en = [cb computeCommandEncoder];
    [en setComputePipelineState:ps];
    NSUInteger tg = ps.maxTotalThreadsPerThreadgroup;
    id<MTLBuffer> b; long result = 0;
    if (!strcmp(mode, "sum")) {
        NSUInteger nt = 1 << 20;
        b = [dev newBufferWithLength:nt * 8 options:MTLResourceStorageModeShared];
        [en setBuffer:b offset:0 atIndex:0]; [en setBytes:&n length:8 atIndex:1];
        [en dispatchThreads:MTLSizeMake(nt, 1, 1) threadsPerThreadgroup:MTLSizeMake(tg, 1, 1)];
    } else if (!strcmp(mode, "map")) {
        b = [dev newBufferWithLength:n * 4 options:MTLResourceStorageModeShared];
        int *xs = b.contents; for (long i = 0; i < n; i++) xs[i] = (int)(i % 1000);
        [en setBuffer:b offset:0 atIndex:0];
        [en dispatchThreads:MTLSizeMake(n, 1, 1) threadsPerThreadgroup:MTLSizeMake(tg, 1, 1)];
    } else {
        uint w = (uint)n;
        b = [dev newBufferWithLength:w * 4 options:MTLResourceStorageModeShared];
        memset(b.contents, 0, w * 4);
        [en setBuffer:b offset:0 atIndex:0]; [en setBytes:&w length:4 atIndex:1];
        [en dispatchThreads:MTLSizeMake(w, w, 1) threadsPerThreadgroup:MTLSizeMake(32, tg / 32, 1)];
    }
    [en endEncoding]; [cb commit]; [cb waitUntilCompleted];
    if (!strcmp(mode, "sum")) { unsigned long long *p = b.contents; for (int t = 0; t < (1 << 20); t++) result += p[t]; result %= 1000000007; }
    else if (!strcmp(mode, "map")) { int *xs = b.contents; for (long i = 0; i < n; i++) result += xs[i]; }
    else { uint *r = b.contents; for (long y = 0; y < n; y++) result += r[y]; }
    fprintf(stderr, "gpu_ms %.2f\n", (cb.GPUEndTime - cb.GPUStartTime) * 1000);
    printf("%ld\n", result);
    }
    return 0;
}
