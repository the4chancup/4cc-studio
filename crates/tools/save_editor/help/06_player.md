The **Player** menu works on the selected player:

- **Set all stats…** gives every ability stat the same value, 40 to 99,
  prefilled with his Attacking Prowess. Weak foot, form and injury are not
  ability stats and stay as they are.
- **Bump all stats…** (Ctrl+B) adds the same amount, -10 to +10, to every
  ability stat, clamped to 40 to 99. The dialog remembers the last amount
  used in this window.
- **Toggle FPC settings** (Ctrl+I) follows the Appearance tab's FPC
  reading: it shows a player that reads "FPC: hidden body" and hides any
  other, the same write as the tab's Show body and Hide body. A player
  hidden by his model's own boots ID reads "body shown", so for him the
  toggle hides again; use Show body. It stays disabled while the
  aesthetics lock is on (see the Appearance tab).
- **Copy stats from…** copies another player's stats onto the selected
  one; **Swap stats with…** trades them. The other player is picked by
  team, then by roster slot. Height, weight, every ability, form, injury,
  weak foot, positions, playing style, skills and COM styles, stronger
  foot and hand, star, playing attitude, the free-kick motion and the
  gameplay edit flags move; names, nationality, age, the other motions and
  every appearance field stay.

Every operation is one undoable step. Set all stats and Bump all stats
write the same fields the Abilities & Skills tab edits, so they set no
edit flag; Copy and Swap stats move the gameplay edit flags with the
stats.

**Generate settings.toml…** and **Generate in-game face…** write the
selected player's look to a player folder of your choice, the files a team
moving to compile-time aesthetics is built from. `settings.toml` holds
everything but the in-game face: the names, the stock boots and gloves IDs,
skin colour, physique, strip and motions. `ingame_face.toml` holds the
in-game face itself: iris colour, face and hair. The save cannot say which
players wear the in-game face, so the action is yours to choose per player,
and neither writes the save. A stripped player is skipped: the game reads
the tables, not his record. When the folder already holds a
`settings.toml`, its comments stay while the values become the record's;
`ingame_face.toml` is written whole.
