#[derive(serde::Serialize, serde::Deserialize)]
pub struct Consumer {
    name: String,
    password: String,
    roles: Vec<String>
}

impl Consumer {
    pub fn new(nm: String, ps: String, rl: Vec<String>) -> Consumer {
        Consumer { name: nm, password: ps, roles: rl }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_password(&self) -> &String {
        &self.password
    }

    pub fn get_roles(&self) -> &Vec<String> {
        &self.roles
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, sqlx::FromRow)]
pub struct ConsumerShow {
    name: String,
    roles: Vec<String>
}

impl ConsumerShow {
    pub fn new(name: String, roles: Vec<String>) -> ConsumerShow {
        ConsumerShow {name: name, roles: roles}
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_roles(&self) -> &Vec<String> {
        &self.roles
    }
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Clone)]
pub struct ConsumerImage {
    image_name: Option<String>,
    image_extension: String,
    image_data: Vec<u8>
}

impl ConsumerImage {
    pub fn new(ign: Option<String>, ige: String, igd: Vec<u8>) -> ConsumerImage {
        ConsumerImage { image_name: ign, image_extension: ige, image_data: igd }
    }

    pub fn get_name(&self) -> &Option<String> {
        &self.image_name
    }

    pub fn get_extension(&self) -> &String {
        &self.image_extension
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.image_data
    }
}

#[derive(serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct ConsumerDisplay {
    #[sqlx(flatten)]
    consumer_show: ConsumerShow,

    #[sqlx(flatten)]
    consumer_image: ConsumerImage
}

impl ConsumerDisplay {
    pub fn new(cs: ConsumerShow, ci: ConsumerImage) -> ConsumerDisplay {
        ConsumerDisplay { consumer_show: cs, consumer_image: ci }
    }

    pub fn get_consumer_show(&self) -> &ConsumerShow {
        &self.consumer_show
    }

    pub fn get_consumer_image(&self) -> &ConsumerImage {
        &self.consumer_image
    }
}
