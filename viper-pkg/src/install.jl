# SPDX-License-Identifier: MPL-2.0

const PACKAGES_DIR = ".viper/packages"

"""
    install_package(pkg::Package)

Install a single package to the local packages directory.
"""
function install_package(pkg::Package)
    # Create packages directory if it doesn't exist
    mkpath(PACKAGES_DIR)

    # Package installation directory
    pkg_dir = joinpath(PACKAGES_DIR, pkg.name, pkg.version)

    # Skip if already installed
    if isdir(pkg_dir)
        println("  ✓ $(pkg.name)@$(pkg.version) (cached)")
        return
    end

    # Download tarball
    if isnothing(pkg.tarball_url)
        error("No tarball URL for package: $(pkg.name)")
    end

    println("  ⬇ $(pkg.name)@$(pkg.version)")

    tarball_path = joinpath(PACKAGES_DIR, "$(pkg.name)-$(pkg.version).tar.gz")

    try
        Downloads.download(pkg.tarball_url, tarball_path)

        # Verify checksum if provided
        if !isnothing(pkg.checksum)
            verify_checksum(tarball_path, pkg.checksum)
        end

        # Extract tarball
        mkpath(pkg_dir)
        Tar.extract(tarball_path, pkg_dir)

        # Cleanup tarball
        rm(tarball_path)

        println("  ✓ $(pkg.name)@$(pkg.version)")
    catch e
        @error "Failed to install package" pkg.name pkg.version exception=e
        rethrow(e)
    end
end

"""
    verify_checksum(file_path::String, expected::String)

Verify SHA-256 checksum of downloaded file.
"""
function verify_checksum(file_path::String, expected::String)
    actual = bytes2hex(sha256(read(file_path)))

    if actual != expected
        error("Checksum mismatch for $file_path\n  Expected: $expected\n  Got: $actual")
    end
end
