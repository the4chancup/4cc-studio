# pes_savefile fixtures

Slices of real saves, never a whole one: a save is 5-11 MB and incompressible once encrypted,
and its serial section is the writing account's Windows SID (plan: `pes_savefile.md` "Container
fixtures"). Extracted by the writing session's `.tmp/save_fixtures.py` from the saves below.
Konami-derived data is Konami's, kept for interoperability (see the repository README).

Per version `NN`:

| File | Holds |
|---|---|
| `pesNN_head.bin` | The file prefix through the description: salt (320) + encrypted header (176 on 16/17, 208 on 18-21) + encrypted description (384); on 15 the seed byte + three MD5 digests + encrypted description (384) + the logo length (4) |
| `pesNN_payload_enc_head.bin` | The first 4096 encrypted bytes of the payload section |
| `pesNN_payload.bin.zz` | The whole decrypted payload, zlib level 9; tests inflate it |

| Version | Source save | Payload / logo / serial (bytes, UTF-16 units) | Notes |
|---|---|---|---|
| 15 | `Tools_4cc/Midcupping/EDIT.bin` (a 4cc PES 15 cup save; identical to the PES 15 KONAMI save on the machine) | 5782808 / 28795 / - | seed byte 195; payload MD5 `c8d4d78bf43317b02b66786b92ce2d0c` equals the head's third digest |
| 16 | `Documents/KONAMI/Pro Evolution Soccer 2016/save/EDIT00000000` | 5886176 / 67218 / 46 | header 176; identifier ends `EDIT` + zeros |
| 17 | `Documents/KONAMI/Pro Evolution Soccer 2017/save/EDIT00000000` | 5266180 / 28689 / 46 | header 176 |
| 18 | `Documents/KONAMI/PRO EVOLUTION SOCCER 2018/save/EDIT00000000` | 5203344 / 66179 / 45 | header 208; game version string `PRO EVOLUTION SOCCER 2018` |
| 19 | `Documents/KONAMI/PRO EVOLUTION SOCCER 2019/<account>/save/EDIT00000000` (identical to `Cups/Genso Cup 2/EDIT00000000`) | 7350036 / 27504 / 46 | header 208; `PRO EVOLUTION SOCCER 2019` |
| 21 | `Documents/KONAMI/eFootball PES 2021 SEASON UPDATE/<account>/save/EDIT00000000` | 10995800 / 14235 / 45 | header 208; `eFootball PES 2021 SEASON UPDATE` |

No PES 20 save exists on the reference machine; the PES 20 key and header size are transcribed
from `masterkey.c`/`crypt.h` and untested until one is found.
