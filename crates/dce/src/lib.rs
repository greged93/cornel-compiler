use bril::types::Block;
use std::collections::{HashMap, HashSet};

/// Returns optimisations on the block for a multi pass of Dead Code Elimination (DCE).
pub fn multi_pass_dce(mut block: Block) -> Block {
    let mut instr_len = 0;

    // Until a single pass of dce doesn't remove code, we keep looping
    while instr_len != block.len() {
        instr_len = block.len();
        block = single_pass_dce(block);
    }

    block
}

/// Returns optimisations on the block for a single pass of Dead Code Elimination (DCE).
/// Also removes assignment of variables which are not used before reassignment.
fn single_pass_dce(mut block: Block) -> Block {
    let mut used = HashMap::new();
    let mut created = HashSet::new();
    let mut remove = HashMap::new();
    let mut prev_index = HashMap::<String, usize, _>::new();

    // Each time a variable is used in an operation, add it to the mapping
    for (index, instr) in block.iter().enumerate() {
        let dest = instr.dest.clone().unwrap_or_default();

        // If the destination is not newly inserted and the used doesn't contain
        // the destination, the variable has been assigned but never used. We register
        // it for deletion.
        if !used.contains_key(&dest) && !created.insert(dest.clone()) {
            let prev_index = prev_index.get(&dest).copied().unwrap_or_default();
            remove.insert(prev_index, true);
        }

        // Insert the destination has being created
        // Add has prev_index
        // Remove from used
        if let Some(d) = instr.dest.as_ref() {
            created.insert(d.clone());
            prev_index.insert(d.clone(), index);
            used.remove(d);
        }

        // Insert the args as being used
        for arg in instr.args.iter() {
            used.insert(arg.clone(), true);
        }
    }

    // Iterate all the instructions, removing assignments to variables that are not used
    let mut index = 0usize;
    block.retain(move |i| {
        if let Some(dest) = i.dest.as_ref() {
            if !used.contains_key(dest) {
                index += 1;
                return false;
            }
        }
        if remove.contains_key(&index) {
            index += 1;
            return false;
        }
        index += 1;

        true
    });

    block
}

#[cfg(test)]
mod tests {
    use super::{multi_pass_dce, single_pass_dce};
    use bril_macros::instruction;

    #[test]
    fn test_single_pass_dce() {
        // Given
        let block = vec![
            instruction!(op = const, value = 1, dest = a),
            instruction!(op = const, value = 2, dest = b),
            instruction!(op = const, value = 2, dest = c),
            instruction!(op = add, args = [a, b], dest = sum),
            instruction!(op = print, args = [sum]),
        ];

        // When
        let optimized_block = single_pass_dce(block);

        // Then
        let expected_block = vec![
            instruction!(op = const, value = 1, dest = a),
            instruction!(op = const, value = 2, dest = b),
            instruction!(op = add, args = [a, b], dest = sum),
            instruction!(op = print, args = [sum]),
        ];

        assert_eq!(optimized_block, expected_block);
    }

    #[test]
    fn test_reassignment_single_pass_dce() {
        // Given
        let block = vec![
            instruction!(op = const, value = 1, dest = a),
            instruction!(op = const, value = 2, dest = a),
            instruction!(op = print, args = [a]),
        ];

        // When
        let optimized_block = single_pass_dce(block);

        // Then
        let expected_block = vec![
            instruction!(op = const, value = 2, dest = a),
            instruction!(op = print, args = [a]),
        ];

        assert_eq!(optimized_block, expected_block);
    }

    #[test]
    fn test_multi_pass_dce() {
        // Given
        let block = vec![
            instruction!(op = const, value = 1, dest = a),
            instruction!(op = const, value = 2, dest = b),
            instruction!(op = const, value = 2, dest = c),
            instruction!(op = const, value = 4, dest = d),
            instruction!(op = add, args = [c, d], dest = sum1),
            instruction!(op = add, args = [a, b], dest = sum2),
            instruction!(op = print, args = [sum2]),
        ];

        // When
        let optimized_block = multi_pass_dce(block);

        // Then
        let expected_block = vec![
            instruction!(op = const, value = 1, dest = a),
            instruction!(op = const, value = 2, dest = b),
            instruction!(op = add, args = [a, b], dest = sum2),
            instruction!(op = print, args = [sum2]),
        ];

        assert_eq!(optimized_block, expected_block);
    }

    #[test]
    fn test_reassignment_multi_pass_dce() {
        // Given
        let block = vec![
            instruction!(op = const, value = 1, dest = a),
            instruction!(op = const, value = 2, dest = b),
            instruction!(op = add, args = [a, b], dest = c),
            instruction!(op = const, value = 3, dest = b),
            instruction!(op = add, args = [a, b], dest = d),
            instruction!(op = print, args = [d]),
        ];

        // When
        let optimized_block = multi_pass_dce(block);

        // Then
        let expected_block = vec![
            instruction!(op = const, value = 1, dest = a),
            instruction!(op = const, value = 3, dest = b),
            instruction!(op = add, args = [a, b], dest = d),
            instruction!(op = print, args = [d]),
        ];

        assert_eq!(optimized_block, expected_block);
    }
}
