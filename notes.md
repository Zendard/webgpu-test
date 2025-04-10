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


RENDER_DISTANCE = 2
(0,0) -> (0,1) : (0,-1) -> (0,2)
(0,2) -> (0,3) : (0,0) -> (0,4)
(0,0) -> (0,-1) : (0,1) -> (0,-2)

RENDER_DISTANCE = 3
(0,0) -> (0,1) : (0,-2) -> (0,3)
(0,2) -> (0,3) : (0,-1) -> (0,5)

new_chunk = previous_chunk + RENDER_DISTANCE * chunk_delta
old_chunk = current_chunk - RENDER_DISTANCE - 1
