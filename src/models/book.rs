use borsh::{BorshDeserialize, BorshSerialize};

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Clone)]
pub struct Book {
    isbn: String,
    title: String,
    author: String,
}

impl Book {
    pub fn new(i: String, t: String, a: String) -> Book {
        Book { isbn: i, title: t, author: a }
    }

    pub fn get_isbn(&self) -> &String {
        &self.isbn
    }

    pub fn get_title(&self) -> &String {
        &self.title
    }

    pub fn get_author(&self) -> &String {
        &self.author
    }
}

#[derive(serde::Deserialize, serde::Serialize, sqlx::FromRow, Clone)]
pub struct BookImage {
    name: String,
    ext: String,
    data: Vec<u8>
}

impl BookImage {
    pub fn new(nm: String, et: String, dt: Vec<u8>) -> BookImage {
        BookImage {name: nm, ext: et, data: dt}
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_extension(&self) -> &String {
        &self.ext
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Clone)]
pub struct BookShow {
    #[sqlx(flatten)]
    book: Book,
    copy_number: i64,
    is_borrowed: bool,
    
    #[sqlx(flatten)]
    book_image: BookImage
}

impl BookShow {
    pub fn new(bk: Book, img_bk: BookImage, cn: i64, isbw: bool) -> BookShow {
        BookShow {book: bk, book_image: img_bk, copy_number: cn, is_borrowed: isbw}
    }

    pub fn get_book(&self) -> &Book {
        &self.book
    }

    pub fn get_image(&self) -> &BookImage {
        &self.book_image
    }

    pub fn get_copy_number(&self) -> i64 {
        i64::clone(&self.copy_number)
    }

    pub fn get_is_borrowed(&self) -> bool {
        bool::clone(&self.is_borrowed)
    }
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Clone)]
pub struct BookShowConsumer {
    #[sqlx(flatten)]
    bookshow: BookShow,
    borrower_id: String,
}

impl BookShowConsumer {
    pub fn new(bks: BookShow, bid: String) -> BookShowConsumer {
        BookShowConsumer { bookshow: bks, borrower_id: bid }
    }

    pub fn get_bookshow(&self) -> &BookShow {
        &self.bookshow
    }

    pub fn get_borrower_id(&self) -> &String {
        &self.borrower_id
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct BookDetails {
    pub search_word: Option<String>,
    pub borrower_id: Option<String>,
    pub copy_number: Option<i64>
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct BookBorrowDetails {
    #[sqlx(flatten)]
    pub book: Book,
    pub borrower_id: String,
    pub copy_number: i64,
    pub days: u64,
}

//book initialize pda
#[derive(BorshDeserialize, BorshSerialize)]
pub struct BookPda {
    pub bump: u8,
    pub isbn: String,
    pub is_borrowed: bool,
    pub borrower: Option<String>,
    pub time: u64,
    pub copy: u64
}

impl BookPda {
    pub fn new(b: u8, i: String, isb: bool, br: Option<String>, t: u64, cp: u64) -> BookPda {
        BookPda { bump: b, isbn: i, is_borrowed: isb, borrower: br, time: t, copy: cp }
    }
    
    pub fn get_bump(&self) -> u8 {
        self.bump
    }
    
    pub fn get_isbn(&self) -> &String {
        &self.isbn
    }
    
    pub fn get_is_borrowed(&self) -> bool {
        self.is_borrowed
    }
    
    pub fn get_borrower(&self) -> Option<&String> {
        self.borrower.as_ref()
    }
    
    pub fn get_borrow_time(&self) -> u64 {
        self.time
    }
    
    pub fn get_which_number_of_copy(&self) -> u64 {
        self.copy
    }
}