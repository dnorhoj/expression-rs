pub fn is_sublist<T: PartialEq>(list: &Vec<T>, sublist: &Vec<T>) -> bool {
    if sublist.is_empty() {
        return true;
    }

    if sublist.len() > list.len() {
        return false;
    }

    for window in list.windows(sublist.len()) {
        if window == sublist {
            return true;
        }
    }

    false
}
