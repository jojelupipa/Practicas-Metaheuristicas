extern crate csv;

// Control de errores
use std::error::Error;
use std::process;

///////////////// ESTRUCTURAS DE DATOS ///////////////////////////

// Template para representar un dato genérico
pub trait DataElem<T> {
    fn new() -> T;
    fn get_num_attributes() -> usize;
    fn get_id(&self) -> i32;
    fn get_class(&self) -> i32;
    fn get_attribute(&self, index: usize) -> f32;
    fn set_id(&mut self, id: i32);
    fn set_class(&mut self, class: i32);
    fn set_attribute(&mut self, index: usize, attr: f32);
}

// Estructura de datos para almacenar las texturas
#[derive(Copy, Clone)]
pub struct TextureRecord {
    id: i32,
    attributes: [f32; 40],
    class: i32,
}

impl DataElem<TextureRecord> for TextureRecord {
    fn new() -> TextureRecord {
        TextureRecord {
            id: -1,
            attributes: [0.0; 40],
            class: -1,
        }
    }

    fn get_num_attributes() -> usize {
        return 40;
    }

    fn get_id(&self) -> i32 {
        return self.id;
    }

    fn get_class(&self) -> i32 {
        return self.class;
    }

    fn get_attribute(&self, index: usize) -> f32 {
        return self.attributes[index];
    }

    fn set_id(&mut self, id: i32) {
        self.id = id;
    }

    fn set_class(&mut self, class: i32) {
        self.class = class;
    }

    fn set_attribute(&mut self, index: usize, attr: f32) {
        self.attributes[index] = attr;
    }
}

/////////////// MÉTODOS DE LOS ALGORITMOS ////////////////////

// TODO: Generalizar donde aparezca TextureRecord a cualquier tipo de
// elemento posible 
/*
fn algoritmo_relief(datos: &Vec<TextureRecord>) {
    let mut pesos = vec![0;TextureRecord::get_num_attributes()];    
}
*/

// Normalizamos los datos de entrada

pub fn normalize_data(data: &mut Vec<TextureRecord>) {
    // Calculamos el máximo y el mínimo para cada atributo  y lo
    // almacenamos en un vector de máximos/mínimos
    let num_attributes = TextureRecord::get_num_attributes();
    let mut maximos = vec![std::f32::MIN; num_attributes];
    let mut minimos = vec![std::f32::MAX; num_attributes];

    for miembro in data.iter() {
        for atributo in 0..num_attributes {
            let valor_actual = miembro.get_attribute(atributo);
            if valor_actual < minimos[atributo] {
                minimos[atributo] = valor_actual;
            }
            if valor_actual > maximos[atributo] {
                maximos[atributo] = valor_actual;
            }
        }
    }

    // Una vez tenemos los máximos/mínimos normalizamos cada atributo
    for miembro in data.iter_mut() {
        for atributo in 0..num_attributes {
            miembro.set_attribute(atributo, (miembro.get_attribute(atributo) - minimos[atributo]) / (maximos[atributo] - minimos[atributo]));
        }
    }
}


// Método principal: Ejecuta el código de la práctica

fn execute()  -> Result<(), Box<Error>> {
    // Reads data, then works with it
    let mut data: Vec<TextureRecord> = Vec::new();
    let mut rdr = csv::Reader::from_path("data/texture.csv")?;

    let mut current_id = 0;
    for result in rdr.records() {
        let mut aux_record = TextureRecord::new();
        let record = result?;

        let mut counter = 0;

        aux_record.id = current_id;

        for field in record.iter() {
            if counter != TextureRecord::get_num_attributes() {
                aux_record.attributes[counter] = field.parse::<f32>().unwrap();
            } else {
                aux_record.class = field.parse::<i32>().unwrap();
            }

            counter += 1;
        }

        current_id += 1;

        data.push(aux_record);
    }

    normalize_data(&mut data);

    //algoritmo_relief(&data);

    
    
    // Debug: Imprimir
    
    /*
    let mut counter2 = 0;
    for data_record in data.iter() {
        println!("Dato {}", counter2);
        
        for miau in 0..TextureRecord::get_num_attributes() {
            println!("Atr {}: {}", miau, data_record.get_attribute(miau));
        }
        counter2 += 1;
    }
     */
     
    
    Ok(())
}

fn main() {
    if let Err(err) = execute() {
        println!("error: {}", err);
        process::exit(1);
    }
}
