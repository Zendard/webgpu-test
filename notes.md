# Problems
## Freeze when copying chunk to GPU
    Only copy new chunk using offsets
    store chunks as HashMap<offset,chunk>

## Falling into ground
    Change on_ground checking function

# New features
## Break blocks
    1. Cast ray and remove block from list
    2. Update visible_faces of surrounding block
