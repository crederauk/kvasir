## 0.2.2 (2021-06-19)

### Fix

- fixed bug where output could write to root output directory and addded unit tests

## 0.2.1 (2021-06-06)

## 0.2.0 (2021-06-06)

### Feat

- added new filters to enable filtering by parser and path
- added new sql file parser

## 0.1.0 (2021-06-05)

### Feat

- added new tera filters and functions
- added new template filters and unit tests
- implemented splitting template output and writing to files
- added command to format parsed json with tera templates
- added command to list currently available parsers
- implemented support for parsing hocon files
- implemented support for xml parsing
- added parsing support for ini files
- Initial commit of working code

### Refactor

- minor refactoring to fix clippy warnings
- removed separate unit test file hierarchy
- improved naming of command line options

### Fix

- improved formatting of log messages
- correctly implemented the --debug flag, changing the log level

### Perf

- improved performance by implementing lazy once loading of file contents
