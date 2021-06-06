## All files
{% for file in files|jsonPath(path="$.*") %}
8<-- output/{{file.parser}}
{{- file.path }}
my custom contents
({{ file.parser }})
{% endfor %}

8<-- output/another-file.txt
File listing:

## Glob Files
{%- for entry in glob(glob="test/resources/*.*") -%}
    {{entry}} {{entry|directory}} {{entry|filename}} {{entry|extension}}
{% endfor %}

## SQL files
{% for file in files|parsed_by(parser="sql") -%}
    {{file.path}}
{% endfor %}


# A specific file
{% set f = files|file(path="test/resources/test.json") %}
* {{ f.0.contents.fruit }} ({{ f.0.contents.size }})