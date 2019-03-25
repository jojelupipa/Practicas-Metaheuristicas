extern crate csv;

// Control de errores
use std::error::Error;
use std::process;

// Diccionarios (usados para mantener distribución de clases uniforme
// en las particiones
use std::collections::HashMap;

// Medidas de tiempo
use std::time::Instant;


///////////////// CONSTANTES /////////////////////////////////////
const NUMERO_PARTICIONES: usize = 5;
const ALPHA_F_EVALUACION: f32 = 0.5;

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

// Algoritmo clasificador 1-NN (asumiendo todos los pesos igual de
// importantes)
//
// Recibe el conjunto de entrenamiento y el de evaluación, clasifica
// los de evaluación en función de la distancia a los primeros
//
// Devuelve una tupla con tasa de clasificación, de reducción (0.0 en
// este caso) y función de evaluación

fn clasificador_1nn(set_entrenamiento: &Vec<TextureRecord>,
                    set_evaluacion: &Vec<TextureRecord>)
                    -> (f32, f32, f32) {
    let mut v_clasificaciones: Vec<i32> = Vec::new(); // TODO: Tal vez
    // esto de error con clases que no sean numéricas

    for miembro in set_evaluacion.iter() {
        let mut clase_vecino_mas_cercano =
            set_entrenamiento[0].get_class();
        let mut distancia_vecino_mas_cercano =
            distancia_entre_vecinos(*miembro, set_entrenamiento[0]);

        for vecino in set_entrenamiento.iter() {
            let distancia = distancia_entre_vecinos(*miembro, *vecino);
            if distancia < distancia_vecino_mas_cercano {
                clase_vecino_mas_cercano = vecino.get_class();
                distancia_vecino_mas_cercano = distancia;
            }
        }
        v_clasificaciones.push(clase_vecino_mas_cercano); 
    }

    // Obtenemos la tupla resultante
    let tasa_clas: f32 = tasa_clasificacion(&set_evaluacion,
                                            &v_clasificaciones);
    let tasa_red = 0.0; // Suponemos que todos los pesos ponderan con
    // 1 y por tanto ninguno es menor que 0.2 y se reduce
    let f_evaluacion = ALPHA_F_EVALUACION * tasa_clas +
        (1.0 - ALPHA_F_EVALUACION) * tasa_red;

    return (tasa_clas, tasa_red, f_evaluacion);
}

fn tasa_clasificacion(set_evaluacion: &Vec<TextureRecord>,
                      v_clasificaciones: &Vec<i32>) -> f32 {
    let mut aciertos = 0.0;

    let mut counter = 0;
    for miembro in set_evaluacion.iter(){
        if miembro.get_class() == v_clasificaciones[counter] {
            aciertos += 1.0;
        }
        counter += 1;
    }
    
    return 100.0 * aciertos / (set_evaluacion.len() as f32);
}

// Hallamos la distancia entre dos elementos vecinos
//
// Recibe los dos elementos a medir
//
// Devuelve la distancia en float

fn distancia_entre_vecinos(elemento1: TextureRecord, elemento2: TextureRecord) -> f32 {
    let num_attributes = TextureRecord::get_num_attributes();
    let mut distancia: f32 = 0.0;
    for atributo in 0..num_attributes {
        let dif  = elemento1.get_attribute(atributo) - elemento2.get_attribute(atributo); 
        distancia += dif * dif;
    }
    distancia = distancia.sqrt();
    
    return distancia;
}

// TODO: Generalizar donde aparezca TextureRecord a cualquier tipo de
// elemento posible (INVESTIGAR TRAITS)

fn algoritmo_relief(datos: &Vec<TextureRecord>) -> Vec<f32> {
    let num_attributes = TextureRecord::get_num_attributes();
    let mut vector_pesos = vec![0.0;num_attributes];

    for miembro in datos.iter() {
        // Buscamos al enemigo y al amigo más cercano
        let mut enemigo_mas_cercano_indice = 0;
        let mut amigo_mas_cercano_indice = 0;
        let mut dist_enemigo_mas_cercano = std::f32::MAX;
        let mut dist_amigo_mas_cercano = std::f32::MAX;

        let mut counter = 0;
        for vecino in datos.iter() {
            if miembro.get_id() != vecino.get_id() { // Comprobamos que no estemos comparando un objeto consigo mismo
                let distancia = distancia_entre_vecinos(*miembro, *vecino);
                // Comprobamos si es "enemigo" y si es mejor que el actual
                if miembro.get_class() != vecino.get_class() {
                    if distancia < dist_enemigo_mas_cercano {
                        dist_enemigo_mas_cercano = distancia;
                        enemigo_mas_cercano_indice = counter;
                    }
                    // Si no es enemigo, es amigo. Comprobamos distancia
                } else {
                    if distancia < dist_amigo_mas_cercano {
                        dist_amigo_mas_cercano = distancia;
                        amigo_mas_cercano_indice = counter;
                    }
                }
            }
            counter += 1;
        }
        // Componente a componente trabajamos con los pesos del vector
        // según la distancia a su mejor amigo y enemigo. Clonamos al
        // amigo y al enemigo para hacer las cuentas
        let amigo_mas_cercano =
            datos[amigo_mas_cercano_indice].clone();
        let enemigo_mas_cercano =
            datos[enemigo_mas_cercano_indice].clone();

        for componente in 0..num_attributes {
            let dist_atributo_amigo =
                (miembro.get_attribute(componente) -
                 amigo_mas_cercano.get_attribute(componente)).abs();
            let dist_atributo_enemigo =
                (miembro.get_attribute(componente) -
                 enemigo_mas_cercano.get_attribute(componente)).abs();
            vector_pesos[componente] += dist_atributo_enemigo -
                                         dist_atributo_amigo;
        }
    }

    // Ahora truncamos los valores negativos a cero y se normalizan
    // los demás dividiendo por el máximo del vector

    let mut maximo = vector_pesos[0];
    for peso in vector_pesos.iter() {
        if *peso > maximo {
            maximo = *peso;
        }
    }

    for peso in vector_pesos.iter_mut() {
        if *peso < 0.0 {
            *peso = 0.0;
        } else {
            *peso /= maximo;
        }
    }

    return vector_pesos;
}

// Crea particiones con distribución de clases uniforme

fn crear_particiones(datos: &Vec<TextureRecord>) -> Vec<Vec<TextureRecord>> {
    let mut particiones: Vec<Vec<TextureRecord>> = Vec::new();
    let mut diccionario_contador_clases = HashMap::new();
    
    for _i in 0..NUMERO_PARTICIONES{
        particiones.push(Vec::new());
    }

    // Usamos un diccionario para contabilizar las clases y procurar
    // una distribución uniforme de estas en todas las particiones
    for muestra in datos.iter() {
        let counter =
            diccionario_contador_clases.entry(muestra.get_class()).or_insert(0);
        particiones[*counter].push(muestra.clone());
        *counter = (*counter + 1) % NUMERO_PARTICIONES;
    }
    
    return particiones;
}


// Normalizamos los datos de entrada

 fn normalizar_datos(datos: &mut Vec<TextureRecord>) {
    // Calculamos el máximo y el mínimo para cada atributo  y lo
    // almacenamos en un vector de máximos/mínimos
    let num_attributes = TextureRecord::get_num_attributes();
    let mut maximos = vec![std::f32::MIN; num_attributes];
    let mut minimos = vec![std::f32::MAX; num_attributes];

    for miembro in datos.iter() {
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
    for miembro in datos.iter_mut() {
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

    normalizar_datos(&mut data);

    let particiones = crear_particiones(&data);

    // Ahora definiremos los conjuntos de entrenamiento y de
    // validación, 5 pares de conjuntos donde cada par estará formado
    // por el 80% de los datos (4/5 particiones) para entrenamiento y
    // 20% (1/5) para validar

    for n_ejecucion in 0..NUMERO_PARTICIONES {
        let mut conjunto_entrenamiento: Vec<TextureRecord> =
            Vec::new();
        let mut conjunto_validacion: Vec<TextureRecord> = Vec::new();

        for particion in 0..NUMERO_PARTICIONES {
            if n_ejecucion != particion {
                conjunto_entrenamiento.extend(&particiones[particion]);
            } else {
                conjunto_validacion = particiones[particion].clone();
            }
        }

        // Utilizamos el clasificador k-nn con k = 1 para evaluar
        // nuestro algoritmo con estos conjuntos de entrenamiento y
        // test

        let mut tiempo_inicial = Instant::now();
        
        let resultados_1nn = clasificador_1nn(&conjunto_entrenamiento,
                                              &conjunto_validacion);
        
        let mut tiempo_total = tiempo_inicial.elapsed().as_millis();

        // Muestra resultados 1nn
        
        println!("Resultados partición: {} ----------", n_ejecucion);
        println!("-- Resultados clasificador 1nn");
        println!("\tTasa de clasificación: {}", resultados_1nn.0);
        println!("\tTasa de reducción: {}", resultados_1nn.1);
        println!("\tFunción de evaluación: {}", resultados_1nn.2);
        println!("\tTiempo de ejecución: {}ms", tiempo_total);

        
    }
    
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
