{% for file in files|jsonPath(path="$.*") %}
    {{- file.path }} ({{ file.parser }})
{% endfor %}