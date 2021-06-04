{% for file in files|jsonPath(path="$.*") %}
8<-- output/{{file.parser}}
{{- file.path }}
my custom contents
({{ file.parser }})
{% endfor %}