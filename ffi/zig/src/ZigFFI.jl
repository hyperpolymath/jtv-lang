# SPDX-License-Identifier: MPL-2.0
"""
    ZigFFI

Julia module for calling Zig libraries via FFI.

# Architecture

Julia → ccall → Zig (C ABI)

# Usage

```julia
using ZigFFI

lib = ZigFFI.load("./libmyzig.so")
result = ZigFFI.call(lib, :add, Cint, (Cint, Cint), 2, 3)
```
"""
module ZigFFI

export load, call, @zig_function

"""
    ZigLibrary

Handle to a loaded Zig shared library.
"""
struct ZigLibrary
    path::String
    handle::Ptr{Cvoid}
end

"""
    load(path::String) -> ZigLibrary

Load a Zig shared library from the given path.
"""
function load(path::String)
    # Resolve relative paths
    full_path = isabspath(path) ? path : abspath(path)

    # Check file exists
    if !isfile(full_path)
        error("Library not found: $full_path")
    end

    # Load using dlopen
    handle = Libdl.dlopen(full_path)
    if handle == C_NULL
        error("Failed to load library: $full_path")
    end

    return ZigLibrary(full_path, handle)
end

"""
    call(lib::ZigLibrary, func::Symbol, rettype, argtypes::Tuple, args...)

Call a function in the Zig library.

# Arguments
- `lib`: The loaded library
- `func`: Function name as symbol
- `rettype`: Return type (e.g., `Cint`)
- `argtypes`: Tuple of argument types
- `args...`: Function arguments
"""
function call(lib::ZigLibrary, func::Symbol, rettype::Type, argtypes::Tuple, args...)
    return ccall(
        Libdl.dlsym(lib.handle, func),
        rettype,
        argtypes,
        args...
    )
end

"""
    @zig_function name(args...)::RetType from "libpath"

Macro to define type-safe wrappers for Zig functions.

# Example
```julia
@zig_function add(a::Int32, b::Int32)::Int32 from "libmyzig"
```
"""
macro zig_function(expr)
    # Parse the expression to extract function signature
    # This is a simplified implementation
    error("@zig_function macro not yet implemented")
end

# Common Zig types mapped to Julia
const ZigInt = Cint
const ZigLong = Clonglong
const ZigFloat = Cfloat
const ZigDouble = Cdouble
const ZigBool = Cuchar
const ZigSize = Csize_t

"""
    version()

Get the ZigFFI module version.
"""
function version()
    return v"0.1.0"
end

# Import Libdl for dynamic loading
using Libdl

end # module
