#![no_main]

use bril_macros::instruction;

instruction!(op = add, args = [a, b, c], dest = sum);
