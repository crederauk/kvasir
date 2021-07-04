## All files
{% for file in files|jsonpath(path="$.*") %}
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
{% for file in files|parsedby(parser="sql") -%}
    {{file.path}}
{% endfor %}


# A specific file
{% for f in files|file(path="test/resources/test.json") -%}
* {{ f.contents.fruit }} ({{ f.contents.size }})
{%- endfor -%}