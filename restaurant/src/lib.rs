mod front_of_house;

pub use crate::front_of_house::hosting;

fn deliver_order() {}

mod back_of_house {
    fn cook_order() {}
    pub fn fix_incorrect_order() {
        cook_order();
        // we can directly refer to names defined in back_of_house
        // deliver_order is a sibling of back_of_house (both children of "create")
        // so we need super:: here to "navigate" from back_of_house to "crate"
        super::deliver_order();
    }

    pub struct Breakfast {
        pub toast: String,
        seasonal_fruit: String,
    }

    impl Breakfast {
        pub fn summer(toast: &str) -> Breakfast {
            Breakfast {
                toast: String::from(toast),
                seasonal_fruit: "peaches".to_string(),
            }
        }
    }

    pub enum Appetizer {
        Soup,
        Salad,
    }
}

pub fn eat_at_restaurant() {
    // absolute path
    crate::front_of_house::hosting::add_to_waitlist();

    // relative path
    // works here because both this fn and the front_of_house module are
    // direct children of the "crate" module
    front_of_house::hosting::seat_at_table();

    // take advantage os "use" clause
    hosting::seat_at_table();

    let mut meal = back_of_house::Breakfast::summer("Rye");
    meal.toast = "Wheat".to_string();
    println!("Please, a {} toast!", meal.toast);

    // doe snot compile because seasonal_fruit is private
    // meal.seasonal_fruit = String::from("blueberries");

    let order1 = back_of_house::Appetizer::Soup;
    let order2 = back_of_house::Appetizer::Salad;
}
