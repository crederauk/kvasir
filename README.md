
# Kvasir
kvasir is a tool for parsing structured text files into JSON format, either
outputting directly to stdout or processing the data through one or more templates
to generate different text-based output formats.

```bash
kvasir document --sources ./test/resources/*.* --templates "-" << EOF

# List all files and parsers
{% for file in files -%}
{{file.path}} {{file.parser}}
{% endfor %}

# List all endpoints in OpenAPI specifications
{% for entry in glob(glob="test/resources/*-api.*") -%}
    {% for file in files|file(path=entry) %}
    {{ file.path }} (v{{ file.contents.info.version }})
        {% for url, attrs in file.contents.paths -%}
        - {{ url }}
        {% endfor -%}
    {% endfor %}

{% endfor %}

EOF
```

Rather than focus on source code file formats (like Java, Python and Go) for which
documentation tools already exist, `kvasir` is intended to parse and document
configuration and human-readable file formats such as YAML, XML, OpenAPI and JSON.
With the ability to parse these formats into a single structure, it's possible to
generate more flexible output and integrate documentation for different formats.

It can be run directly or as a pre-processor within CI/CD pipelines to generate and
embed documentation into markdown files, READMEs or other documentation tools.

## Motivation
There are many documentation generation tools for projects, but most are either generic
(e.g. Sphinx) or specific to one type of file (e.g. Swagger). Kvasir exists in the space
between the two, allowing generic documentation generators to be supplemented with information
for specific file types. Customising the templates allows for any type of text-based
file to be generated.

Because Kvasir parses all files and types into a single JSON data structure, this means
that one can use [JsonPath expressions](https://github.com/json-path/JsonPath) to display,
filter and generate documentation in a way that makes sense for the project. For larger
projects, perhaps those that use a mono-repository and have many different types of development
artefacts, using this approach means that a single, consistent set of documentation can
be created with documentation for different types of artefacts interspersed.

## Build status
The `cargo` tool is used to build Kvasir, generating a standalone, statically-compiled binary
that can be deployed into a container or build environment.

## Code style
Kvasir follows the standard `rustfmt` styling.

## Usage
Kvasir runs as a command-line application. Available commands can be listed with:
`kvasir --help`.


Examples:
```bash
    # Parse multiple input files to JSON and output to stdout
    kvasir parse --sources /path/to/**/*.yaml /path/to/**/*.xml
    
    # Parse multiple input files to JSON and format them with the specified templates
    kvasir document --sources /path/to/**/*.yaml --templates templates/base.tpl --root-template base.tpl
```

## Tests
Run tests with `cargo test`.

## License
Kvasir is Apache 2 licensed, © 2021 Andrew James & Credera.
