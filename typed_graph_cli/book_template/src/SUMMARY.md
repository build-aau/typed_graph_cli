# Summary

- [Primitives](./primitives.md)

# Heads
{% for section in main_sections -%}
- [{{ section.title }}]({{ section.path }})
    - [Imports]({{ section.title }}/imports.md)
    {%- for stm in section.imports -%}
        - [{{ stm.title }}]({{ section.title }}/{{ stm.path }})
    {%- endfor %}
    - [Nodes]({{ section.title }}/nodes.md)
    {%- for stm in section.nodes %}
        - [{{ stm.title }}]({{ section.title }}/{{ stm.path | replace(from="\", to="/") }})
    {%- endfor %}
    - [Edges]({{ section.title }}/edges.md)
    {%- for stm in section.edges %}
        - [{{ stm.title }}]({{ section.title }}/{{ stm.path | replace(from="\", to="/") }})
    {%- endfor %}
    - [Structs]({{ section.title }}/structs.md)
    {%- for stm in section.structs %}
        - [{{ stm.title }}]({{ section.title }}/{{ stm.path | replace(from="\", to="/") }})
    {%- endfor %}
    - [Enum]({{ section.title }}/types.md)
    {%- for stm in section.types %}
        - [{{ stm.title }}]({{ section.title }}/{{ stm.path | replace(from="\", to="/") }})
    {%- endfor -%}
{% endfor -%}

---
# Other
{% for section in other_sections -%}
- [{{ section.title }}]({{ section.path }})
    - [Imports]({{ section.title }}/imports.md)
    {%- for stm in section.imports -%}
        - [{{ stm.title }}]({{ section.title }}/{{ stm.path | replace(from="\", to="/") }})
    {%- endfor %}
    - [Nodes]({{ section.title }}/nodes.md)
    {%- for stm in section.nodes %}
        - [{{ stm.title }}]({{ section.title }}/{{ stm.path | replace(from="\", to="/") }})
    {%- endfor %}
    - [Edges]({{ section.title }}/edges.md)
    {%- for stm in section.edges %}
        - [{{ stm.title }}]({{ section.title }}/{{ stm.path | replace(from="\", to="/") }})
    {%- endfor %}
    - [Structs]({{ section.title }}/structs.md)
    {%- for stm in section.structs %}
        - [{{ stm.title }}]({{ section.title }}/{{ stm.path | replace(from="\", to="/") }})
    {%- endfor %}
    - [Enum]({{ section.title }}/types.md)
    {%- for stm in section.types %}
        - [{{ stm.title }}]({{ section.title }}/{{ stm.path | replace(from="\", to="/") }})
    {%- endfor -%}
{% endfor -%}