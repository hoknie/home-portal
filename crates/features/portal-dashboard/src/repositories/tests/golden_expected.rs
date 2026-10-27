pub const EXPECTED: [(&str, &str); 14] = [
    (
        "implicit_unchanged",
        r##"# the home page
[network]
port = 8080

# counts first
[[dashboard.widgets]]
widget = "status-summary"

[[dashboard.widgets]]
widget = "riga"
width = 6

[[dashboard.library]]
id = "status-summary"
type = "status-summary"

# the weather
[[dashboard.library]]
id = "riga"
type = "weather"
settings = { latitude = 56.95, longitude = 24.11 }
"##,
    ),
    (
        "implicit_reordered_and_resized",
        r##"# the home page
[network]
port = 8080

[[dashboard.widgets]]
widget = "riga"
width = 4
height = 2

# counts first
[[dashboard.widgets]]
widget = "status-summary"

[[dashboard.library]]
id = "status-summary"
type = "status-summary"

# the weather
[[dashboard.library]]
id = "riga"
type = "weather"
settings = { latitude = 56.95, longitude = 24.11 }
"##,
    ),
    (
        "implicit_becomes_sections",
        r##"# the home page
[network]
port = 8080

# counts first
[[dashboard.widgets]]
widget = "status-summary"

[[dashboard.widgets]]
widget = "riga"
width = 6

[[dashboard.library]]
id = "status-summary"
type = "status-summary"

# the weather
[[dashboard.library]]
id = "riga"
type = "weather"
settings = { latitude = 56.95, longitude = 24.11 }

[[dashboard.sections]]
id = "main"
title = "Main"
"##,
    ),
    (
        "sectioned_unchanged",
        r##"[[dashboard.sections]]
id = "now"
title = "Now"

# later things
[[dashboard.sections]]
id = "later"

# no section of its own
[[dashboard.widgets]]
widget = "status-summary"

[[dashboard.widgets]]
widget = "riga"
section = "later"

[[dashboard.library]]
id = "status-summary"
type = "status-summary"

[[dashboard.library]]
id = "riga"
type = "weather"
"##,
    ),
    (
        "sectioned_widget_without_section_moved",
        r##"[[dashboard.sections]]
id = "now"
title = "Now"

# later things
[[dashboard.sections]]
id = "later"

# no section of its own
[[dashboard.widgets]]
widget = "status-summary"
column = 3

[[dashboard.widgets]]
widget = "riga"
section = "later"

[[dashboard.library]]
id = "status-summary"
type = "status-summary"

[[dashboard.library]]
id = "riga"
type = "weather"
"##,
    ),
    (
        "sectioned_edit_omits_widget",
        r##"[[dashboard.sections]]
id = "now"
title = "Now"

# later things
[[dashboard.sections]]
id = "later"

# no section of its own
[[dashboard.widgets]]
widget = "status-summary"

[[dashboard.widgets]]
widget = "riga"
section = "later"
row = 5

[[dashboard.library]]
id = "status-summary"
type = "status-summary"

[[dashboard.library]]
id = "riga"
type = "weather"
"##,
    ),
    (
        "sectioned_new_placement",
        r##"[[dashboard.sections]]
id = "now"
title = "Now"

# no section of its own
[[dashboard.widgets]]
widget = "status-summary"

[[dashboard.widgets]]
widget = "riga"
section = "later"

[[dashboard.widgets]]
widget = "riga"

[[dashboard.library]]
id = "status-summary"
type = "status-summary"

[[dashboard.library]]
id = "riga"
type = "weather"
"##,
    ),
    (
        "inline_migrated",
        r##"[[dashboard.library]]
id = "weather"
type = "weather"

# an inline clock
[[dashboard.widgets]]
widget = "clock"

# an inline weather without an id
[[dashboard.widgets]]
widget = "weather-2"

[[dashboard.widgets]]
widget = "weather"

[[dashboard.library]]
id = "clock"
type = "clock"
title = "Time"

[[dashboard.library]]
id = "weather-2"
type = "weather"
settings = { latitude = 1.5 }
"##,
    ),
    (
        "absent_dashboard",
        r##"[network]
port = 8080

[[dashboard.library]]
id = "status-summary"
type = "status-summary"

[[dashboard.library]]
id = "services"
type = "services"

[[dashboard.widgets]]
widget = "services"
"##,
    ),
    (
        "library_new_derived",
        r##"weather
---
# the home page
[network]
port = 8080

# counts first
[[dashboard.widgets]]
widget = "status-summary"

[[dashboard.widgets]]
widget = "riga"
width = 6

[[dashboard.library]]
id = "status-summary"
type = "status-summary"

# the weather
[[dashboard.library]]
id = "riga"
type = "weather"
settings = { latitude = 56.95, longitude = 24.11 }

[[dashboard.library]]
id = "weather"
type = "weather"
"##,
    ),
    (
        "library_new_given",
        r##"disks
---
# the home page
[network]
port = 8080

# counts first
[[dashboard.widgets]]
widget = "status-summary"

[[dashboard.widgets]]
widget = "riga"
width = 6

[[dashboard.library]]
id = "status-summary"
type = "status-summary"

# the weather
[[dashboard.library]]
id = "riga"
type = "weather"
settings = { latitude = 56.95, longitude = 24.11 }

[[dashboard.library]]
id = "disks"
type = "custom"
title = "Disks"
width = 4
"##,
    ),
    (
        "library_replaced",
        r##"riga
---
# the home page
[network]
port = 8080

# counts first
[[dashboard.widgets]]
widget = "status-summary"

[[dashboard.widgets]]
widget = "riga"
width = 6

[[dashboard.library]]
id = "status-summary"
type = "status-summary"

# the weather
[[dashboard.library]]
id = "riga"
type = "weather"
title = "Riga"
"##,
    ),
    (
        "library_new_after_migration",
        r##"clock-2
---
[[dashboard.library]]
id = "weather"
type = "weather"

# an inline clock
[[dashboard.widgets]]
widget = "clock"

# an inline weather without an id
[[dashboard.widgets]]
widget = "weather-2"

[[dashboard.widgets]]
widget = "weather"

[[dashboard.library]]
id = "clock"
type = "clock"
title = "Time"

[[dashboard.library]]
id = "weather-2"
type = "weather"
settings = { latitude = 1.5 }

[[dashboard.library]]
id = "clock-2"
type = "clock"
"##,
    ),
    (
        "library_removed_after_migration",
        r##"
# an inline clock
[[dashboard.widgets]]
widget = "clock"

# an inline weather without an id
[[dashboard.widgets]]
widget = "weather-2"

[[dashboard.widgets]]
widget = "weather"

[[dashboard.library]]
id = "clock"
type = "clock"
title = "Time"

[[dashboard.library]]
id = "weather-2"
type = "weather"
settings = { latitude = 1.5 }
"##,
    ),
];
