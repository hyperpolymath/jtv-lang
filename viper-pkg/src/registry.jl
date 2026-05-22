# SPDX-License-Identifier: MPL-2.0

const REGISTRY_URL = "https://packages.julia-the-viper.dev/api/v1"
const FALLBACK_MODE = :git  # Use git until registry deployed

"""
    fetch_package(name::String, version::String="latest")

Fetch package metadata from registry or git.
"""
function fetch_package(name::String, version::String="latest")
    if FALLBACK_MODE == :git
        return fetch_from_git(name, version)
    else
        return fetch_from_registry(name, version)
    end
end

"""
    search_packages(query::String, limit::Int=20)

Search for packages in registry.
"""
function search_packages(query::String, limit::Int=20)
    if FALLBACK_MODE == :git
        return search_git(query, limit)
    else
        return search_registry(query, limit)
    end
end

# Registry mode (future)

function fetch_from_registry(name::String, version::String)
    url = "$REGISTRY_URL/packages/$name"

    try
        response = HTTP.get(url, status_exception=false)

        if response.status != 200
            return nothing
        end

        data = JSON3.read(response.body)
        target_version = version == "latest" ? data.latest_version : version

        return Package(
            data.name,
            target_version,
            get(data, :description, nothing),
            get(data, :repository, nothing),
            get(data, :dependencies, Dict{String,String}()),
            get(data, :tarball_url, nothing),
            get(data, :checksum, nothing)
        )
    catch e
        @warn "Failed to fetch package" name exception=e
        return nothing
    end
end

function search_registry(query::String, limit::Int)
    url = "$REGISTRY_URL/packages?q=$query&limit=$limit"

    try
        response = HTTP.get(url)
        data = JSON3.read(response.body)

        return [Package(
            pkg.name,
            pkg.latest_version,
            get(pkg, :description, nothing),
            get(pkg, :repository, nothing),
            Dict{String,String}(),
            nothing,
            nothing
        ) for pkg in data]
    catch e
        @warn "Failed to search packages" query exception=e
        return Package[]
    end
end

# Git fallback mode (current)

function fetch_from_git(name::String, version::String)
    # Try common git hosting patterns
    urls = [
        "https://github.com/hyperpolymath/$name",
        "https://gitlab.com/hyperpolymath/$name",
        "https://git.sr.ht/~hyperpolymath/$name"
    ]

    for base_url in urls
        pkg = try_fetch_git(base_url, name, version)
        if !isnothing(pkg)
            return pkg
        end
    end

    return nothing
end

function try_fetch_git(base_url::String, name::String, version::String)
    # Try to fetch Viper.toml
    branch = version == "latest" ? "main" : version
    manifest_url = "$base_url/raw/$branch/Viper.toml"

    try
        response = HTTP.get(manifest_url, status_exception=false)

        if response.status != 200
            return nothing
        end

        manifest = TOML.parse(String(response.body))
        pkg_info = get(manifest, "package", Dict())

        return Package(
            get(pkg_info, "name", name),
            get(pkg_info, "version", version),
            get(pkg_info, "description", nothing),
            base_url,
            get(manifest, "dependencies", Dict{String,String}()),
            "$base_url/archive/$branch.tar.gz",
            nothing  # No checksum in git mode
        )
    catch e
        # Silently fail for this URL, try next
        return nothing
    end
end

function search_git(query::String, limit::Int)
    # Git search not yet implemented
    Package[]
end
