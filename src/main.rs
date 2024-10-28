use cursive::theme::Theme;
use cursive::style::{Color,BaseColor::*, PaletteColor::*};
use cursive::traits::{Nameable, Resizable};
use cursive::view::Scrollable;
use cursive::views::{Dialog, EditView, ListView};
use cursive::{Cursive, CursiveExt};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Read;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
    product_type: String,
    quantity: usize,
    price_per_unit: f64,
    sales_tax: f64,
    total_price: f64,
}
// database file
const FILE_NAME: &str = "products.json";

//custon terminal theme
fn custom_theme_from_cursive(siv: &Cursive) -> Theme {
    // We'll return the current theme with a small modification.
    let mut theme = siv.current_theme().clone();

    theme.palette[Background] = Color::TerminalDefault;
    theme.palette[View] = Cyan.dark();
        theme.palette[Primary] = Yellow.light();
        theme.palette[TitlePrimary] = Red.light();
        theme.palette[Highlight] = Green.dark();
        theme.palette[HighlightText] = White.light();

    theme
}


//save product to file
fn save_product_to_file(products: &Vec<Product>) -> io::Result<()> {
    let file: File = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(FILE_NAME)?;
    serde_json::to_writer(&file, &products)?;

    Ok(())
}

fn load_product_from_file(file_name: &str) -> Vec<Product> {
    if let Ok(mut file) = File::open(file_name) {
        let mut data = String::new();
        if file.read_to_string(&mut data).is_ok() {
            if let Ok(products) = serde_json::from_str::<Vec<Product>>(&data) {
                return products;
            }
        }
    }
    Vec::new()
}

fn main() {
    let mut app = Cursive::default();

    let theme = custom_theme_from_cursive(&app);

    app.set_theme(theme);

    app.set_window_title("Inventory Store");


    let products = Arc::new(Mutex::new(load_product_from_file(FILE_NAME)));

    app.add_layer(
        Dialog::new()
            .title("Inventory Store")
            .content(
                ListView::new()
                    .child("Product Type", EditView::new().with_name("product_type"))
                    .child("Quantity", EditView::new().with_name("quantity"))
                    .child("Price per unit",EditView::new().with_name("price_per_unit"),),
            )
            .button("Save", {
                let product_clone = Arc::clone(&products);
                move |s| {
                    let product_type = s
                        .call_on_name("product_type", |v: &mut EditView| v.get_content())
                        .unwrap()
                        .to_string();

                    let quantity = s
                        .call_on_name("quantity", |v: &mut EditView| v.get_content())
                        .unwrap()
                        .parse::<usize>()
                        .unwrap_or(0);

                    let price_per_unit = s
                        .call_on_name("price_per_unit", |v: &mut EditView| v.get_content())
                        .unwrap()
                        .parse::<f64>()
                        .unwrap_or(0.0);

                    if product_type.is_empty() {
                        s.add_layer(
                            Dialog::info("Product type is empty").title("Empty product type"),
                        );
                        return;
                    }
                    if quantity == 0 {
                        s.add_layer(
                            Dialog::info("Quantity cannot be zero").title("Invalid quantity"),
                        );
                        return;
                    }
                    if price_per_unit == 0.0 {
                        s.add_layer(
                            Dialog::info("Price per unit cannot be zero")
                           .title("Invalid price"),
                        );
                        return;
                    }
                    let sales_tax = 0.10 * price_per_unit;
                    let total_price = quantity as f64 * price_per_unit + sales_tax;
                    let new_product = Product {
                        product_type,
                        quantity,
                        price_per_unit,
                        sales_tax,
                        total_price,
                    };
                    let mut products = product_clone.lock().unwrap();
                    products.push(new_product.clone());

                    if let Err(err) = save_product_to_file(&products) {
                        s.add_layer(
                            Dialog::info(format!("Error saving product: {}", err))
                                .title("Save Error"),
                        );
                    } else {
                        s.add_layer(
                            Dialog::info("Product saved successfully")
                           .title("Save successfully"),
                        );
                    }

                    // s.pop_layer();
                }
            }).button("Show All", {
            let products = Arc::clone(&products);
            move |s| {
                let products = products.lock().unwrap();
                let mut content = String::new();
                for (index, product) in products.iter().enumerate() {
                    content.push_str(&format!("Product ID: {}.\nItem: {}\nQuantity: {}\nPrice: {}\nSales tax: {}\nTotal price: {}\n\n",
                                              index + 1, product.product_type, product.quantity, product.price_per_unit, product.sales_tax, product.total_price));
                }
                if content.is_empty() {
                    content.push_str("No products found");
                }
                s.add_layer(Dialog::info(content)
                    .title("All Products")
                    .scrollable()
                    .fixed_size((30, 30))
                    );
            }
        }).button("Delete By Id", {
            move |s| {
                let id = EditView::new()
                    .with_name("delete_id")
                    .fixed_width(10);
                s.add_layer(Dialog::new()
                    .title("Delete product")
                    .content(ListView::new()
                        .child("Enter Product Id to delete", id))
                    .button("Confirm", {
                        let product_clone = Arc::clone(&products);
                        move |s| {
                            let id = s.call_on_name("delete_id", |v: &mut EditView| {
                                v.get_content()
                            }).unwrap().to_string();
                           if id.is_empty(){
                                s.add_layer(
                                    Dialog::info("Id cannot be empty")
                                        .title("Empty ID"),
                                );
                                return;
                           }
                            if let Ok(id) = id.parse::<usize>() {
                                let mut products = product_clone.lock().unwrap();
                                if id > 0 && id <= products.len() {
                                    products.remove(id - 1);
                                    if let Err(err) = save_product_to_file(&products) {
                                        s.add_layer(
                                            Dialog::info(format!("Error deleting product: {}", err)).
                                           title("Delete Error"),
                                        );
                                    } else {
                                        s.add_layer(
                                            Dialog::info("Product Delete successfully")
                                           .title("Delete successfully"),
                                        );
                                    }
                                } else {
                                    s.add_layer(
                                        Dialog::info("Invalid product ID").title("Invalid ID"),
                                    );
                                }
                            }
                        }
                    })
                    .button("Cancel", |s| {
                        s.pop_layer();
                    })
                );
            }
        })
            .button("Quit", |s| {
                s.quit();
            })
    );

    app.add_global_callback('q', |s| s.quit());

    app.run();
}

// cursive = "0.21.1"
// serde = { version = "1.0.213", features = ["derive"] }
// serde_json = "1.0.132"
