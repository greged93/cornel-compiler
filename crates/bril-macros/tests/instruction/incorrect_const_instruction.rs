#![no_main]

use bril_macros::instruction;

instruction!(op = const, value = 1, args = [a]);
