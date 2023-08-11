# Led Controller
> A tool to control my Led Strip

## Arguments

- Set Effect (-s --set-effect)
    Sets the [effect](#effects)
- Kill (-k --kill)
    Kills the daemon
- Web (-w --web)
    Sets the status of [led.rugmj.dev](https://led.rugmj.dev)
- Test (-t --test)
    Tests the led strip Red -> Green -> Blue -> White -> Repeat

## Effects
> Effects are WIP

- Rainbow
    A simple rainbow effect using sine
- Random
    A strobe like effect which picks random colours for each block (10) leds
- Test
    The test effect (Should be set with -t instead)

## Dbus Control
Everything the cli can control can be controlled using dbus

An example command to set the effect to the Rainbow effect: `busctl --user call dev.rugmj.LedController /dev/rugmj/LedController dev.rugmj.LedController1 SetEffect 0`
