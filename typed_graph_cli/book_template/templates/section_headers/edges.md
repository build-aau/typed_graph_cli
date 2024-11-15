# Edge
## JSON Representation
```json
{
    "weight": {
        /* EdgeType */: {
            "id": /* EdgeId */,
            /* Edge fields */
        }
    },
    "source": /* NodeId */,
    "target": /* NodeId */
}
```

## Available EdgeTypes

{% for edge in edges %}
- [{{ edge }}](edges/{{ edge }}.md)
{% endfor %}