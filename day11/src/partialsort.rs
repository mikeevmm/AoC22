use std::fmt::Debug;

/// Partially sort the given slice from largest to smallest item, ensuring sorted order only for
/// elements of index ≤ k.
pub fn partial_sort<T: Ord>(slice: &mut [T], k: usize) {
    partial_sort_recursive(slice, k, 0)
}

fn partial_sort_recursive<T: Ord>(slice: &mut [T], k: usize, i: usize) {
    // Quick sort, but discarding ordering to the right of pivots @ index k or greater.
    // See https://en.m.wikipedia.org/wiki/Partial_sorting#Specialised_sorting_algorithms
    if slice.len() < 2 {
        return;
    }
    if slice.len() == 2 {
        if slice[0] < slice[1] {
            slice.swap(0, 1);
        }
        return;
    }
    let mut left = 0;
    let mut right = slice.len() - 1;
    let partition = slice.len() / 2;
    let mut pivot = slice.len() / 2;
    loop {
        while left < right && slice[left] > slice[pivot] {
            left += 1;
        }
        while left < right && slice[right] < slice[pivot] {
            right -= 1;
        }
        if left >= right {
            break;
        }
        slice.swap(left, right);
        if left == pivot {
            pivot = right;
        } else if right == pivot {
            pivot = left;
        }
    }
    partial_sort_recursive(&mut slice[0..=pivot], k, i);
    if i + pivot < k {
        partial_sort_recursive(&mut slice[(pivot + 1)..], k, i + partition + 1);
    }
}

#[test]
fn partial_sort_test() {
    let mut x = [5, 1, 2, 9, 7, 6, 3, 1];
    partial_sort(&mut x, 3);
    println!("{:?}", x);
}