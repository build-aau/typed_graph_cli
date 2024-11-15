# Node
## JSON Representation
```json
{
    /* NodeType */: {
        "id": /* NodeId */,
        /* Node fields */
    }
}
```

## Available NodeTypes

{% for node in nodes %}
- [{{ node }}](nodes/{{ node }}.md)
{% endfor %}