# SPDX-License-Identifier: MPL-2.0
# config.ncl - JTV Playground Configuration
# Type-safe configuration using Nickel
# See: https://nickel-lang.org

{
  # Project metadata
  project = {
    name = "jtv-playground",
    version = "1.0.0",
    description = "Playground for JTV Playground",
    license = "MPL-2.0",
    author = "Jonathan D.A. Jewell",
    repository = "https://github.com/hyperpolymath/jtv-playground",
  },

  # Documentation configuration
  docs = {
    readme = "README.adoc",
  },

  # Development settings
  development = {
    default_tasks = [
      "build",
      "lint",
      "fmt",
      "test",
    ],
  },

  # Computed values
  full_name = project.name ++ " v" ++ project.version,
}
