#![no_main]

use bril_macros::instruction;

instruction!(op = print, args = [a], value = 1);
