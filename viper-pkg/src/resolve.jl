# SPDX-License-Identifier: MPL-2.0

"""
    resolve_dependencies(deps::Dict{String,String})

Resolve dependency graph for a set of direct dependencies.
Returns a flat list of all packages to install (including transitive deps).

Uses a simple topological sort for now. Future: Implement PubGrub algorithm.
"""
function resolve_dependencies(deps::Dict{String,String})
    resolved = Package[]
    visited = Set{String}()

    for (name, version) in deps
        resolve_recursive(name, version, resolved, visited)
    end

    return resolved
end

function resolve_recursive(name::String, version::String, resolved::Vector{Package}, visited::Set{String})
    # Skip if already processed
    if name in visited
        return
    end

    # Fetch package metadata
    pkg = fetch_package(name, version)
    if isnothing(pkg)
        error("Could not resolve package: $name@$version")
    end

    # Mark as visited
    push!(visited, name)

    # Recursively resolve dependencies
    for (dep_name, dep_version) in pkg.dependencies
        resolve_recursive(dep_name, dep_version, resolved, visited)
    end

    # Add to resolved list after dependencies
    push!(resolved, pkg)
end
