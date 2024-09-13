//! Contains the implementation of the Local Value Numbering algorithm.

use bril::types::{Block, Operation};
use eyre::eyre;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub fn local_value_numbering(mut block: Block) -> eyre::Result<Block> {
    let mut var2num = HashMap::new();
    let mut num2var = Vec::new();
    let mut lvn = HashMap::new();
    let mut num = 0usize;

    for i in block.iter_mut() {
        // Handle the id instruction in a special case
        if i.op == Operation::Id {
            let a = i.args.first().ok_or(eyre!("missing argument for Id"))?;
            let num = var2num
                .get(a)
                .copied()
                .ok_or(eyre!("missing {a} in var2num"))?;
            var2num.insert(
                i.dest.clone().ok_or(eyre!("missing destination for Id"))?,
                num,
            );
            i.args = vec![num2var
                .get(num)
                .cloned()
                .ok_or(eyre!("missing {num} in num2var"))?];
            continue;
        }

        // We convert the arguments and the value if any into their number in the var2num mapping.
        // This converts the expression to something like (add, 1, 2) or (const 42).
        let value_arr = i.value.iter().map(|x| *x as usize).collect::<Vec<_>>();
        let args_num = i
            .args
            .iter()
            .map(|a| {
                var2num
                    .get(a)
                    .copied()
                    .ok_or(eyre!("missing {a} in var2num"))
            })
            .collect::<eyre::Result<Vec<_>>>()?;
        let mut args = [args_num.clone(), value_arr].concat();
        args.sort();
        let expression = (i.op.clone(), args);

        let dest = i.dest.clone().unwrap_or_default();
        let entry = lvn.entry(expression);

        match entry {
            // If vacant, update the var2num and num2var, increase num
            // and insert the new expression in the mapping.
            // Also retrieve the new arguments from the var2num
            // mapping
            Entry::Vacant(v) => {
                var2num.insert(dest.clone(), num);
                num2var.push(dest.clone());
                v.insert((dest, num));
                i.args = args_num
                    .into_iter()
                    .map(|arg| {
                        num2var
                            .get(arg)
                            .cloned()
                            .ok_or(eyre!("missing {arg} in num2var"))
                    })
                    .collect::<eyre::Result<Vec<_>>>()?;
                num += 1;
            }
            // If occupied, retrieve the expression number from
            // the lvn mapping and point the destination of the
            // opcode towards this number. Also update the instruction
            // to use [`bril::types::Operation::Id`]
            Entry::Occupied(e) => {
                let (var, n) = e.get();
                var2num.insert(dest, *n);
                i.op = bril::types::Operation::Id;
                i.args = vec![var.clone()];
            }
        };
    }

    Ok(block)
}

#[cfg(test)]
mod tests {
    use super::local_value_numbering;
    use bril_macros::instruction;

    #[test]
    fn test_local_value_numbering() {
        // Given
        let block = vec![
            instruction!(op = const, value = 1, dest = a),
            instruction!(op = const, value = 2, dest = b),
            instruction!(op = add, args = [a, b], dest = sum1),
            instruction!(op = add, args = [a, b], dest = sum2),
            instruction!(op = mul, args = [sum1, sum2], dest = prod),
            instruction!(op = print, args = [prod]),
        ];

        // When
        let optimized_block = local_value_numbering(block).expect("failed to apply lvn");

        // Then
        let expected_block = vec![
            instruction!(op = const, value = 1, dest = a),
            instruction!(op = const, value = 2, dest = b),
            instruction!(op = add, args = [a, b], dest = sum1),
            instruction!(op = id, args = [sum1], dest = sum2),
            instruction!(op = mul, args = [sum1, sum1], dest = prod),
            instruction!(op = print, args = [prod]),
        ];

        assert_eq!(optimized_block, expected_block);
    }

    #[test]
    fn test_local_value_numbering_commutativity() {
        // Given
        let block = vec![
            instruction!(op = const, value = 1, dest = a),
            instruction!(op = const, value = 2, dest = b),
            instruction!(op = add, args = [a, b], dest = sum1),
            instruction!(op = add, args = [b, a], dest = sum2),
            instruction!(op = mul, args = [sum1, sum2], dest = prod),
            instruction!(op = print, args = [prod]),
        ];

        // When
        let optimized_block = local_value_numbering(block).expect("failed to apply lvn");

        // Then
        let expected_block = vec![
            instruction!(op = const, value = 1, dest = a),
            instruction!(op = const, value = 2, dest = b),
            instruction!(op = add, args = [a, b], dest = sum1),
            instruction!(op = id, args = [sum1], dest = sum2),
            instruction!(op = mul, args = [sum1, sum1], dest = prod),
            instruction!(op = print, args = [prod]),
        ];

        assert_eq!(optimized_block, expected_block);
    }

    #[test]
    fn test_local_value_numbering_constant_propagation() {
        // Given
        let block = vec![
            instruction!(op = const, value = 1, dest = x),
            instruction!(op = id, args = [x], dest = copy1),
            instruction!(op = id, args = [copy1], dest = copy2),
            instruction!(op = id, args = [copy2], dest = copy3),
            instruction!(op = print, args = [copy3]),
        ];

        // When
        let optimized_block = local_value_numbering(block).expect("failed to apply lvn");

        // Then
        let expected_block = vec![
            instruction!(op = const, value = 1, dest = x),
            instruction!(op = id, args = [x], dest = copy1),
            instruction!(op = id, args = [x], dest = copy2),
            instruction!(op = id, args = [x], dest = copy3),
            instruction!(op = print, args = [x]),
        ];

        assert_eq!(optimized_block, expected_block);
    }
}
