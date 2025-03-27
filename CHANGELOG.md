# Changelog

## [Unreleased]

### Fixed
- Fixed parameter substitution when parameters have descriptions (e.g., `@param:Description`). 
  The description part was not being properly removed from the command after substitution.
- Fixed an unused assignment warning in the `prompt_parameters` function.

### Added
- Added debug logging to help troubleshoot parameter substitution. 