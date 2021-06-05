{% for file in files|jsonPath(path="$.*") %}
8<-- output/{{file.parser}}
{{- file.path }}
my custom contents
({{ file.parser }})
{% endfor %}

8<-- output/another-file.txt
File listing:

{%- for entry in glob(glob="test/resources/*.*") -%}
    {{entry}} {{entry|directory}} {{entry|filename}} {{entry|extension}}
{% endfor %}