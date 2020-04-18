# web: buttons are hidden, but keyboard can still be used

The split button may be hidden from view when a pair is not being displayed,
but the player can still hit "P" on their keyboard to choose split.

# Consider NoAuto save stats option that still loads stats from disk

To NoAuto, perhaps. There is now a `save` command the user can use, and
`--save-stats Never` may lead to confusion about whether the `save` command
does anything.

Or maybe add a NoAuto option. Never currently also means we don't touch the
filesystem for PlayStats-related reasons
