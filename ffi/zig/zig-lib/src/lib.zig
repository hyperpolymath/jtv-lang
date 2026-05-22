// SPDX-License-Identifier: MPL-2.0
// Julia-Zig-FFI - Bidirectional FFI between Julia and Zig
//
// Directions:
// 1. Julia -> Zig: Direct ccall to exported functions
// 2. Zig -> Julia: Callback functions via cfunction pointers

const std = @import("std");

// ============================================================================
// VERSION INFO
// ============================================================================

pub const VERSION_MAJOR: u32 = 0;
pub const VERSION_MINOR: u32 = 1;
pub const VERSION_PATCH: u32 = 0;

export fn get_version() callconv(.C) u32 {
    return (VERSION_MAJOR << 16) | (VERSION_MINOR << 8) | VERSION_PATCH;
}

// ============================================================================
// BASIC FUNCTIONS (Julia -> Zig)
// ============================================================================

export fn add(a: i64, b: i64) callconv(.C) i64 {
    return a + b;
}

export fn multiply(a: i64, b: i64) callconv(.C) i64 {
    return a * b;
}

export fn factorial(n: u64) callconv(.C) u64 {
    if (n <= 1) return 1;
    var result: u64 = 1;
    var i: u64 = 2;
    while (i <= n) : (i += 1) {
        result *= i;
    }
    return result;
}

export fn fibonacci(n: u64) callconv(.C) u64 {
    if (n <= 1) return n;
    var a: u64 = 0;
    var b: u64 = 1;
    for (2..n + 1) |_| {
        const temp = a + b;
        a = b;
        b = temp;
    }
    return b;
}

export fn string_length(str: [*:0]const u8) callconv(.C) usize {
    return std.mem.len(str);
}

// ============================================================================
// ARRAY OPERATIONS (Useful for Julia's numerical computing)
// ============================================================================

/// Sum array elements
export fn array_sum(ptr: [*]const f64, len: usize) callconv(.C) f64 {
    var sum: f64 = 0;
    for (ptr[0..len]) |val| {
        sum += val;
    }
    return sum;
}

/// Compute dot product
export fn dot_product(a: [*]const f64, b: [*]const f64, len: usize) callconv(.C) f64 {
    var result: f64 = 0;
    for (0..len) |i| {
        result += a[i] * b[i];
    }
    return result;
}

/// Scale array in place
export fn array_scale(ptr: [*]f64, len: usize, factor: f64) callconv(.C) void {
    for (ptr[0..len]) |*val| {
        val.* *= factor;
    }
}

// ============================================================================
// CALLBACK SUPPORT (Zig -> Julia)
// ============================================================================

/// Callback types matching Julia's cfunction signatures
pub const FloatCallback = *const fn (f64) callconv(.C) f64;
pub const IntCallback = *const fn (i64) callconv(.C) void;
pub const ProgressCallback = *const fn (usize, usize) callconv(.C) bool;

/// Callback storage
var g_float_callback: ?FloatCallback = null;
var g_int_callback: ?IntCallback = null;
var g_progress_callback: ?ProgressCallback = null;

/// Register callbacks
export fn register_float_callback(cb: FloatCallback) callconv(.C) void {
    g_float_callback = cb;
}

export fn register_int_callback(cb: IntCallback) callconv(.C) void {
    g_int_callback = cb;
}

export fn register_progress_callback(cb: ProgressCallback) callconv(.C) void {
    g_progress_callback = cb;
}

/// Map operation using Julia callback
export fn map_array(
    src: [*]const f64,
    dst: [*]f64,
    len: usize,
    transform: FloatCallback,
) callconv(.C) void {
    for (0..len) |i| {
        dst[i] = transform(src[i]);
    }
}

/// Reduce operation using Julia callback
export fn reduce_array(
    ptr: [*]const f64,
    len: usize,
    initial: f64,
    reducer: *const fn (f64, f64) callconv(.C) f64,
) callconv(.C) f64 {
    var acc = initial;
    for (ptr[0..len]) |val| {
        acc = reducer(acc, val);
    }
    return acc;
}

/// Process with progress reporting
export fn process_with_progress(
    data: [*]f64,
    len: usize,
    progress_cb: ProgressCallback,
) callconv(.C) f64 {
    var sum: f64 = 0;
    for (0..len) |i| {
        sum += data[i];
        // Report progress every 100 items
        if (i % 100 == 0) {
            if (!progress_cb(i, len)) {
                break; // Cancelled
            }
        }
    }
    _ = progress_cb(len, len); // Final progress
    return sum;
}

// ============================================================================
// TESTS
// ============================================================================

test "basic arithmetic" {
    try std.testing.expectEqual(@as(i64, 5), add(2, 3));
    try std.testing.expectEqual(@as(i64, 6), multiply(2, 3));
}

test "array operations" {
    const data = [_]f64{ 1.0, 2.0, 3.0, 4.0, 5.0 };
    try std.testing.expectEqual(@as(f64, 15.0), array_sum(&data, data.len));
}

test "dot product" {
    const a = [_]f64{ 1.0, 2.0, 3.0 };
    const b = [_]f64{ 4.0, 5.0, 6.0 };
    try std.testing.expectEqual(@as(f64, 32.0), dot_product(&a, &b, 3));
}
