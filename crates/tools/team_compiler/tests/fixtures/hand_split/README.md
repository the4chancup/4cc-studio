# Hand auto-split fixture (step 4.18)

Written by the lead; the generator is kept under `scripts/provenance/fixtures/`.

| File | Contents | For |
|---|---|---|
| `body.fmdl` | a full-body model reduced to one strip, both arms with their hands on: 11 columns (x = -5..5) by 3 rows, 33 vertices, 40 faces, one mesh, one material (`body_mat`, no textures of its own; the export added the dummy normal and specular maps). Columns -1..1 weigh on `sk_chest`, 2 on `sk_forearm_l`, 3 on `sk_hand_l`, 4 half on `sk_hand_l` and half on `skh_index_mcp_l`, 5 on `skh_index_mcp_l`; the negative columns the same with `_r`. Built as `model_convert` IR and written by `ir_to_fmdl` (`hand_split_body.rs`); `fmdl::check` finds nothing. Split (import without a skeleton, `split_by_skeleton_group`): each glove 9 vertices and 8 faces (columns 3 to 5 of its side), the body 21 vertices and 24 faces (columns -3 to 3); 39 vertices in all, the 6 of columns 3 and -3 in the body and a glove both, and each of the 40 faces in exactly one part | TC-MOD-31 |
