# autoclicke-rs
An advanced cli autoclicker written in rust for Linux. Works on X11 (not tested) and Wayland. And on any installation that has uinput enabled.

# Features
This is just an output of the --help command.
```
This autoclicker can either work when holding down a mouse key, always when on and with mouse keys togging it on and off.

Usage: autoclicker [OPTIONS] [MOUSE_PATH]

Arguments:
  [MOUSE_PATH]
          Will autoclick based on this mouse

Options:
  -l, --left-click-delay <LEFT_CLICK_DELAY>
          Delay between clicks (default) or target cps
          
          [default: 0.05]

  -r, --right-click-delay <RIGHT_CLICK_DELAY>
          Delay between clicks (default) or target cps
          
          [default: 0.05]

  -f, --fast-click-delay <FAST_CLICK_DELAY>
          Fast mode can be enabled for left clicking. Delay between clicks (default) or target cps
          
          [default: 0.02]

  -c, --click-mode <CLICK_MODE>
          In what format are the input delays given. Use the same delay parameters in any case. A delay under 1000 nanoseconds will not scale linearly and will cap at around 17k cps. You can get faster by putting the click delay to zero

          Possible values:
          - delay: Delay between clicks
          - cps:   Target cps
          
          [default: delay]

  -m, --mode <MODE>
          Program operation mode

          Possible values:
          - hold:   Clicks when holding the button down
          - toggle: Toggles clicking so you don't have to hold anything to click. Start delay determines whether hold or toggle mode is used
          - both:   Why not just have both? Hold and toggle
          - always: Instantly starts to spam enabled keys before the program is killed
          
          [default: hold]

  -d, --disable-on-click
          On toggle mode if you click, it will stop the autoclicker

      --enable-left
          Start left click enabled

      --enable-right
          Start right click enabled

      --enable-fast
          Start fast mode enabled

      --start-delay-left <START_DELAY_LEFT>
          Delay before to start left clicking
          
          [default: 0.1]

      --start-delay-right <START_DELAY_RIGHT>
          Delay before to start right clicking
          
          [default: 0.1]

      --randomize
          Whether to randomize the delay slightly

      --deviation <DEVIATION>
          How much can the calculated random can differ from base. In float percentage. Allowed range: 0 to 1
          
          [default: 0.3]

  -s, --scroll-changes-cps
          Change cps when autoclicking by scrolling. Left click takes priority when you're clicking with both. Will reset after stopping clicking. Does not work with always mode

      --factor <FACTOR>
          Factor of how much the scroll changes the delay
          
          [default: 1.1]

      --minimum-delay <MINIMUM_DELAY>
          Minimum delay allowed when scrolling. If fast mode is enabled, this is ignored. If input mode is cps this will be the max cps
          
          [default: 0]

  -u, --update-delay <UPDATE_DELAY>
          Interface update delay in milliseconds
          
          [default: 10]

      --spammers <SPAMMERS>
          How many spammers to spawn when activating the autoclicker. This is like a multiplier for cps. I will not take any responsibility for changing this parameter
          
          [default: 1]

  -d, --debug
          Print useful information for debugging. (Not fully ready)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
