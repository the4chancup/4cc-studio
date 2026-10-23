Unlimited Tool Works 0.0.1
==========================

A test version, for the testers of DBC and VGL. If anything goes wrong or reads strangely,
tell us: that is what this version is for.

In the window, the Save editor and the Ruleset editor are on the left bar. The Team compiler
and the Export upgrader are greyed out there: in this version they run through the two
launchers below.


The first time
--------------

1. Unpack this archive into a folder of its own, anywhere you like.
2. Open UnlimitedToolWorks.exe. It asks where to keep its settings: either answer works.
3. On the left bar, pick your PES. If the PES box turns yellow or red, click the gear at the
   bottom of the bar (Settings) and set "PES folder" to the folder that holds the game's
   PES20xx.exe.
   Then close the window.


Upgrading old exports
---------------------

Exports made for Red need upgrading once before they compile here.

1. Put the old exports in the old_exports folder: folders, .zip or .7z files.
2. Drag your PES savefile onto quick_upgrade.bat. The savefile is the file named
   EDIT00000000, in the game's save folder under Documents\KONAMI.
3. The upgraded exports land in the exports folder. The window says how many were upgraded;
   for one that was not, it says why above that line.
4. Then move the upgraded old exports out of old_exports. An export already upgraded is not
   upgraded again: the next run lists it as not upgraded ("the output folder already
   exists") and leaves its folder in exports as it is.


Compiling
---------

1. Put the exports to compile in the exports folder (the upgraded ones are already there).
2. Close PES, then double-click quick_compile.bat. It compiles every export in the exports
   folder and installs the result in the game.
3. Read the report, then press a key to close the window.


When something goes wrong
-------------------------

Every message names a code, such as export_disabled. The help explains each one: open
UnlimitedToolWorks.exe and click the ? at the bottom of the left bar.

If a problem stops you, compile with Red as usual, and tell us what happened.
