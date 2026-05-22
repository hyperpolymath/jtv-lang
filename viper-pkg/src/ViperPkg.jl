# SPDX-License-Identifier: MPL-2.0
module ViperPkg

using HTTP, JSON3, TOML, SHA, Downloads, Tar

export install, resolve, search, add_package, remove_package, update_packages

include("types.jl")
include("registry.jl")
include("resolve.jl")
include("install.jl")

"""
    install(manifest_path::String="Viper.toml")

Install all dependencies from a Viper.toml manifest file.

# Example
```julia
ViperPkg.install()  # Uses ./Viper.toml
ViperPkg.install("myproject/Viper.toml")
```
"""
function install(manifest_path::String="Viper.toml")
    if !isfile(manifest_path)
        error("Manifest file not found: $manifest_path")
    end

    manifest = TOML.parsefile(manifest_path)
    deps = get(manifest, "dependencies", Dict{String,String}())

    if isempty(deps)
        println("No dependencies to install")
        return
    end

    println("Resolving dependencies...")
    resolved = resolve_dependencies(deps)

    println("Installing $(length(resolved)) packages...")
    for pkg in resolved
        install_package(pkg)
    end

    println("✓ All packages installed successfully")
end

"""
    add_package(name::String, version::String="latest")

Add a package to the current project.
"""
function add_package(name::String, version::String="latest")
    manifest_path = "Viper.toml"

    # Create manifest if it doesn't exist
    if !isfile(manifest_path)
        create_default_manifest(manifest_path)
    end

    # Fetch package metadata
    println("Fetching $name...")
    pkg = fetch_package(name, version)

    if isnothing(pkg)
        error("Package not found: $name")
    end

    # Add to manifest
    manifest = TOML.parsefile(manifest_path)
    if !haskey(manifest, "dependencies")
        manifest["dependencies"] = Dict{String,String}()
    end
    manifest["dependencies"][name] = pkg.version

    # Write manifest
    open(manifest_path, "w") do io
        TOML.print(io, manifest)
    end

    # Install package
    println("Installing $name@$(pkg.version)...")
    install_package(pkg)

    println("✓ Added $name@$(pkg.version)")
end

"""
    remove_package(name::String)

Remove a package from the current project.
"""
function remove_package(name::String)
    manifest_path = "Viper.toml"

    if !isfile(manifest_path)
        error("No Viper.toml found")
    end

    manifest = TOML.parsefile(manifest_path)
    deps = get(manifest, "dependencies", Dict{String,String}())

    if !haskey(deps, name)
        error("Package not in dependencies: $name")
    end

    delete!(deps, name)
    manifest["dependencies"] = deps

    # Write manifest
    open(manifest_path, "w") do io
        TOML.print(io, manifest)
    end

    println("✓ Removed $name")
end

"""
    search(query::String; limit::Int=20)

Search for packages in the registry.
"""
function search(query::String; limit::Int=20)
    results = search_packages(query, limit)

    if isempty(results)
        println("No packages found matching: $query")
        return
    end

    println("Found $(length(results)) packages:")
    for pkg in results
        println("  $(pkg.name) ($(pkg.version))")
        if !isnothing(pkg.description)
            println("    $(pkg.description)")
        end
    end
end

"""
    update_packages()

Update all packages to their latest versions.
"""
function update_packages()
    manifest_path = "Viper.toml"

    if !isfile(manifest_path)
        error("No Viper.toml found")
    end

    manifest = TOML.parsefile(manifest_path)
    deps = get(manifest, "dependencies", Dict{String,String}())

    if isempty(deps)
        println("No dependencies to update")
        return
    end

    println("Checking for updates...")
    updated = 0

    for (name, current_version) in deps
        pkg = fetch_package(name, "latest")
        if !isnothing(pkg) && pkg.version != current_version
            println("  Updating $name: $current_version → $(pkg.version)")
            deps[name] = pkg.version
            updated += 1
        end
    end

    if updated > 0
        manifest["dependencies"] = deps
        open(manifest_path, "w") do io
            TOML.print(io, manifest)
        end

        println("✓ Updated $updated packages")
        println("Run `ViperPkg.install()` to install updates")
    else
        println("✓ All packages are up to date")
    end
end

function create_default_manifest(path::String)
    manifest = Dict(
        "package" => Dict(
            "name" => basename(dirname(abspath(path))),
            "version" => "0.1.0",
            "authors" => ["Unknown"],
            "license" => "MPL-2.0"
        ),
        "dependencies" => Dict{String,String}()
    )

    open(path, "w") do io
        TOML.print(io, manifest)
    end
end

end # module
