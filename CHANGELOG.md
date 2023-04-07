# Changelog

All notable changes to this project will be documented in this file.

## [0.5.0] - 2023-04-07

- [Breaking] Commands previously using `serial` argument now takes `&Option<S: ToString>` instead of `Option<String>`. (#8)
- Adds `serial` argument for `host-feature` command. (#8)

Thanks @jagenheim for contributing !
