// Leer csv
extern crate csv;

// Control de errores
use std::error::Error;
use std::process;

// Diccionarios (usados para mantener distribución de clases uniforme
// en las particiones
use std::collections::HashMap;

// Medidas de tiempo
use std::time::Instant;

// Generador números aleatorios
extern crate rand;
use rand::distributions::{Distribution, Normal, Uniform};
//use rand::thread_rng;
use rand::seq::SliceRandom; // Para poder mezclar con shuffle
use rand::prelude::*;

// Manejo argumentos (para indicar la semilla)
use std::env;

///////////////// CONSTANTES /////////////////////////////////////
const NUMERO_PARTICIONES: usize = 5;
const ALPHA_F_OBJETIVO: f32 = 0.5;
const MAXIMO_EVALUACIONES_F_OBJ: usize = 15000;
const VARIANZA_MUTACIONES: f64 = 0.3;
const PROB_CRUCE_AGG: f32 = 0.7;
const PROB_MUTACION: f32 = 0.001;
const PROB_BUSQUEDA_MEMETICO: f32 = 0.1;
const BLX_VALUE: f64 = 0.3;

const TAM_POBLACION_GEN: usize = 30;
const TAM_POBLACION_MEM: usize = 10;
const PADRES_ESTACIONARIO: usize = 2;

const CARACTERISTICAS_IONOSFERA: usize = 34;
const CARACTERISTICAS_TEXTURA: usize = 40;
const CARACTERISTICAS_COLPOSCPIA: usize = 62;

///////////////// ESTRUCTURAS DE DATOS ///////////////////////////

// Interfaz para trabajar con un tipo de dato genérico
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
    attributes: [f32; CARACTERISTICAS_TEXTURA],
    class: i32,
}

// Implementación de la interfaz DataElem<T> para TextureRecord
impl DataElem<TextureRecord> for TextureRecord {
    fn new() -> TextureRecord {
        TextureRecord {
            id: -1,
            attributes: [0.0; CARACTERISTICAS_TEXTURA],
            class: -1,
        }
    }

    fn get_num_attributes() -> usize {
        return CARACTERISTICAS_TEXTURA;
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

// Estructura de datos para almacenar las texturas
#[derive(Copy, Clone)]
pub struct IonosphereRecord {
    id: i32,
    attributes: [f32; CARACTERISTICAS_IONOSFERA],
    class: i32,
}

// Implementación de la interfaz DataElem<T> para IonosphereRecord
impl DataElem<IonosphereRecord> for IonosphereRecord {
    fn new() -> IonosphereRecord {
        IonosphereRecord {
            id: -1,
            attributes: [0.0; CARACTERISTICAS_IONOSFERA],
            class: -1,
        }
    }

    fn get_num_attributes() -> usize {
        return CARACTERISTICAS_IONOSFERA;
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

// Estructura de datos para almacenar las colposcopias
#[derive(Copy, Clone)]
pub struct ColposcopyRecord {
    id: i32,
    attributes: [f32; CARACTERISTICAS_COLPOSCPIA],
    class: i32,
}

// Implementación de la interfaz DataElem<T> para ColposcopyRecord
impl DataElem<ColposcopyRecord> for ColposcopyRecord {
    fn new() -> ColposcopyRecord {
        ColposcopyRecord {
            id: -1,
            attributes: [0.0; CARACTERISTICAS_COLPOSCPIA],
            class: -1,
        }
    }

    fn get_num_attributes() -> usize {
        return CARACTERISTICAS_COLPOSCPIA;
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

// Enum para indicar el tipo de cruce, blx o aritmético

#[derive(PartialEq)]
enum VarianteCruce {
    BLX,
    ARIT,
}

// Enum para indicar la variante del alg memético
#[derive(PartialEq)]
enum VarianteMemetico {
    TODOS_CROMOSOMAS,
    CROM_ALEATORIO_P0_1,
    MEJORES_N_CROMOSOMAS,
}

/////////////// MÉTODOS DE LOS ALGORITMOS ////////////////////

// Algoritmo clasificador 1-NN (asumiendo todos los pesos igual de
// importantes)
//
// Recibe el conjunto de entrenamiento y el de evaluación, clasifica
// los de evaluación en función de la distancia a los primeros
//
// Devuelve una tupla con tasa de clasificación, de reducción (0.0 en
// este caso) y función objetivo

fn clasificador_1nn<T: DataElem<T> + Copy + Clone>(
    set_entrenamiento: &Vec<T>,
    set_evaluacion: &Vec<T>)
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
    let f_objetivo = ALPHA_F_OBJETIVO * tasa_clas +
        (1.0 - ALPHA_F_OBJETIVO) * tasa_red;

    return (tasa_clas, tasa_red, f_objetivo);
}


// Algoritmo clasificador 1-NN (con pesos independientes)
//
// Recibe el conjunto de entrenamiento y el de evaluación, clasifica
// los de evaluación en función de la distancia a los primeros
//
// Devuelve una tupla con tasa de clasificación, de reducción 
// y función objetivo


fn clasificador_1nn_con_pesos<T: DataElem<T> + Copy + Clone>(
    set_entrenamiento: &Vec<T>,
    set_evaluacion: &Vec<T>,
    v_pesos: &Vec<f32>)
    -> (f32, f32, f32) {

    let mut v_clasificaciones: Vec<i32> = Vec::new(); // TODO: Tal vez
    // esto de error con clases que no sean numéricas

    let mut pesos_red = v_pesos.clone();
    let mut n_reducidos = 0.0;
   
    for p in pesos_red.iter_mut() {
        if *p < 0.2 {
            *p = 0.0;
            n_reducidos += 1.0;
        }
    }
    
    for miembro in set_evaluacion.iter() {
        let mut clase_vecino_mas_cercano =
            set_entrenamiento[0].get_class();
        let mut distancia_vecino_mas_cercano =
            distancia_ponderada_entre_vecinos(*miembro,
        set_entrenamiento[0], &pesos_red);

        for vecino in set_entrenamiento.iter() {
            if miembro.get_id() != vecino.get_id() { // En caso de que
                // set_entrenamiento = set_evaluacion
                let distancia =
                    distancia_ponderada_entre_vecinos(*miembro, *vecino,
                                                      &pesos_red);
                if distancia < distancia_vecino_mas_cercano {
                    clase_vecino_mas_cercano = vecino.get_class();
                    distancia_vecino_mas_cercano = distancia;
                }
            }
        }
        v_clasificaciones.push(clase_vecino_mas_cercano); 
    }

    // Obtenemos la tupla resultante
    let tasa_clas: f32 = tasa_clasificacion(&set_evaluacion,
                                            &v_clasificaciones);
    let tasa_red: f32 = tasa_reduccion(n_reducidos,
                                       pesos_red.len() as f32); 
    let f_objetivo = funcion_objetivo(tasa_clas, tasa_red);

    return (tasa_clas, tasa_red, f_objetivo);
}

fn tasa_clasificacion<T: DataElem<T> + Copy + Clone>(
    set_evaluacion: &Vec<T>,
    v_clasificaciones: &Vec<i32>)
    -> f32 {
    
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

fn tasa_reduccion(
    n_reducidos: f32,
    n_caracteristicas: f32)
    -> f32 {

    return 100.0 * n_reducidos / n_caracteristicas;
}

fn funcion_objetivo(
    tasa_clas: f32,
    tasa_red: f32)
    -> f32 {

    return ALPHA_F_OBJETIVO * tasa_clas + (1.0 - ALPHA_F_OBJETIVO) *
        tasa_red; 
}

// Hallamos la distancia entre dos elementos vecinos
//
// Recibe los dos elementos a medir
//
// Devuelve la distancia en float

fn distancia_entre_vecinos<T: DataElem<T> + Copy + Clone>(
    elemento1: T,
    elemento2: T)
    -> f32 {
    
    let num_attributes = T::get_num_attributes();
    let mut distancia: f32 = 0.0;
    for atributo in 0..num_attributes {
        let dif  = elemento1.get_attribute(atributo) - elemento2.get_attribute(atributo); 
        distancia += dif * dif;
    }
    distancia = distancia.sqrt();
    
    return distancia;
}

fn distancia_ponderada_entre_vecinos<T: DataElem<T> + Copy + Clone>(
    elemento1: T,
    elemento2: T,
    pesos: &Vec<f32>)
    -> f32 {
    
    let num_attributes = T::get_num_attributes();
    let mut distancia: f32 = 0.0;
    for atributo in 0..num_attributes {
        let dif  = (elemento1.get_attribute(atributo) -
                    elemento2.get_attribute(atributo))
            * pesos[atributo]; 
        distancia += dif * dif;
    }
    distancia = distancia.sqrt();
    
    return distancia;
}

//////////////////////////////////////////////////
/////////// ALGORITMOS UTILIZADOS ////////////////
//////////////////////////////////////////////////

// Algoritmo Relief (Greedy)

fn algoritmo_relief<T: DataElem<T> + Copy + Clone>(
    datos: &Vec<T>)
    -> Vec<f32> {
    
    let num_attributes = T::get_num_attributes();
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

// Búsqueda Local

fn busqueda_local<T: DataElem<T> + Copy + Clone>(
    datos: &Vec<T>,
    seed_u64: u64)
    -> Vec<f32> {
    let num_attributes = T::get_num_attributes();
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed_u64); // Para generar números aleatorios

    // Generamos el vector aleatorio inicial
    let mut pesos: Vec<f32> = vec![0.0; num_attributes];
    let distribucion_uniforme = Uniform::new(0.0, 1.0);
    let distribucion_normal = Normal::new(0.0, VARIANZA_MUTACIONES);
    for atributo in 0..num_attributes {
        pesos[atributo] = distribucion_uniforme.sample(&mut rng);
    }

    // Generamos un vector de índices y lo desordenamos para
    // proporcionar aleatoriedad en el proceso de mejora de un
    // atributo (peso de una característica) al azar
    let mut indices: Vec<usize> = (0..num_attributes).collect();
    indices.shuffle(&mut rng);
    
    let mut n_mutaciones = 0;
    let mut n_vecinos_gen_sin_mejorar = 0;
    let max_vecinos_gen_sin_mejorar = 20 * num_attributes;

    // Comprobamos la calidad de estos pesos 
    let mut mejor_f_obj = clasificador_1nn_con_pesos(&datos, &datos,
                                                  &pesos).2;

    //println!("F obj inicial: {}", mejor_f_obj);

    while n_vecinos_gen_sin_mejorar < max_vecinos_gen_sin_mejorar &&
        n_mutaciones < MAXIMO_EVALUACIONES_F_OBJ {
            let mut pesos_aux = pesos.clone();

            if indices.is_empty() {
                indices = (0..num_attributes).collect();
                indices.shuffle(&mut rng);
            }

            let indice_a_mejorar = indices.pop().expect("Vector vacío");

            pesos_aux[indice_a_mejorar] +=
                distribucion_normal.sample(&mut rng) as f32;
            if pesos_aux[indice_a_mejorar] < 0.0 {
                pesos_aux[indice_a_mejorar] = 0.0;
            } else if pesos_aux[indice_a_mejorar] > 1.0 {
                pesos_aux[indice_a_mejorar] = 1.0;
            }

            let f_obj_actual = clasificador_1nn_con_pesos(&datos,
                                                          &datos, &pesos_aux).2;

            if f_obj_actual > mejor_f_obj {
                pesos = pesos_aux;
                mejor_f_obj = f_obj_actual;
                n_vecinos_gen_sin_mejorar = 0;

                // Resetear indices
                indices = (0..num_attributes).collect();
                indices.shuffle(&mut rng);
                //debug
                //println!("Vector de pesos mejorado. F_obj: {}",
                //mejor_f_obj);
            } else {
                n_vecinos_gen_sin_mejorar += 1;
            }
            n_mutaciones += 1;
        }    
    
    return pesos;
    
}


// Algoritmo genético estacionario con remplazo elitista
fn alg_genetico_generacional_elitista<T: DataElem<T> + Copy + Clone>(
    datos: &Vec<T>,
    seed_u64: u64,
    variante_cruce: VarianteCruce)
    -> Vec<f32> {
    
    let num_attributes = T::get_num_attributes();
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed_u64); // Para generar números aleatorios con una semilla
    let mut pesos: Vec<f32> = vec![0.0; num_attributes];

    // Distribución para las mutaciones
    let distribucion_normal = Normal::new(0.0, VARIANZA_MUTACIONES); 
        
    // Generar población inicial
    let mut poblacion: Vec<Vec<f32>> =
        Vec::with_capacity(TAM_POBLACION_GEN);

    inicializar_poblacion(&mut poblacion, &mut rng, num_attributes,
    TAM_POBLACION_GEN); 

    // Mientras no se cumpla la condición de parada: 15000
    // evaluaciones
    let mut contador_evaluaciones: usize = 0;
    //let mut generacion = 0;
    while contador_evaluaciones < MAXIMO_EVALUACIONES_F_OBJ {

        // Buscamos cuál es la mejor solución de la población actual
        // para mantenerla posteriormente (pues así lo requiere el
        // elitismo. Al evaluar todos los individuos aprovechamos para
        // meterlos en una estructura formada por un par que contengan
        // el cromosoma en sí y su f_obj para ahorrarnos repetir la
        // evaluación en inmediatamente posteriores
        let mut pob_evaluada: Vec<(Vec<f32>, f32)> =
            Vec::with_capacity(TAM_POBLACION_GEN);
        let mut mejor_cromosoma = 0;
        pob_evaluada.push(
            (
                poblacion[0].clone(),
                clasificador_1nn_con_pesos(&datos,
                                           &datos,
                                           &poblacion[0]).2
            )
        );
        for i in 0..poblacion.len() {
            if i != mejor_cromosoma {
                pob_evaluada.push(
                    (
                        poblacion[i].clone(),
                        clasificador_1nn_con_pesos(&datos,
                                                   &datos,
                                                   &poblacion[i]).2
                    )
                );

                if pob_evaluada[i].1 > pob_evaluada[mejor_cromosoma].1
                {
                    mejor_cromosoma = i;
                }        
            }
        }
        
        contador_evaluaciones += TAM_POBLACION_GEN;

        //// Seleccionamos padres mediante torneo binario
        
        let mut seleccionados: Vec<(Vec<f32>, f32)> =
            Vec::with_capacity(TAM_POBLACION_GEN * 2);

        let tam_poblacion_padres = TAM_POBLACION_GEN * 2;
        while seleccionados.len() < tam_poblacion_padres {
            let candidato1 = rng.gen_range(0, TAM_POBLACION_GEN);
            let candidato2 = rng.gen_range(0, TAM_POBLACION_GEN);

            let ganador = torneo_binario(candidato1,
                                        candidato2,
                                        &pob_evaluada);
            seleccionados.push(pob_evaluada[ganador].clone());
        }

        // DEBUG: Mostrar padres seleccionados pre mutaciones
        //println!("Evaluaciones: {}. Tamaño pob padres: {}", contador_evaluaciones ,seleccionados.len());
        // for i in 0..seleccionados.len() {
        //     println!("Cromosoma: {} , fitness: {}", i, seleccionados[i].1);
        // }
        // println!();
        
        //// Recombinamos la población, mutamos, reemplazamos la población
        //// a partir de la población anterior y los nuevos hijos

        // Cruce. Actuamos en función de si es cruce aritmético o BLX

       
        let mut pob_provisional: Vec<(Vec<f32>, f32)> =
            Vec::with_capacity(TAM_POBLACION_GEN);

        let n_cruces: usize = ((tam_poblacion_padres as f32) *
                                   PROB_CRUCE_AGG).trunc() as usize;
            
        if variante_cruce == VarianteCruce::ARIT {
            cruce_aritmetico(&datos,
                             &seleccionados,
                             &mut pob_provisional,
                             n_cruces,
                             num_attributes);
            contador_evaluaciones += n_cruces;
            
        } else if variante_cruce == VarianteCruce::BLX {
            cruce_blx(&datos,
                      &seleccionados,
                      &mut pob_provisional,
                      n_cruces,
                      num_attributes,
                      &mut rng);
            contador_evaluaciones += n_cruces;
        }

        // Completamos la población con padres de la anterior
        // generación
        let mut counter = n_cruces; // Introducimos desde el
        // último candidato a reproducirse hasta que se llene la
        // población
        while pob_provisional.len() < TAM_POBLACION_GEN {
            pob_provisional.push(
                (seleccionados[counter].0.clone(),
                 seleccionados[counter].1
                )
            );
            counter += 1;
        }

        // Mutamos el número de genes esperado
        let mutaciones_esperadas = (num_attributes as f32 *
                                    TAM_POBLACION_GEN as f32 *
                                    PROB_MUTACION
        ).trunc() as usize;

        let mut mut_realizadas = 0;
        while mut_realizadas < mutaciones_esperadas {
            let cromosoma_mut = rng.gen_range(0, TAM_POBLACION_GEN);
            let gen_mut = rng.gen_range(0, num_attributes);

            let mut pesos_aux =
            pob_provisional[cromosoma_mut].0.clone(); 
            
            pesos_aux[gen_mut] +=
                distribucion_normal.sample(&mut rng) as f32;
            if pesos_aux[gen_mut] < 0.0 {
                pesos_aux[gen_mut] = 0.0;
            } else if pesos_aux[gen_mut] > 1.0 {
                pesos_aux[gen_mut] = 1.0;
            }

            pob_provisional.remove(cromosoma_mut);
            pob_provisional.push(
                (pesos_aux.clone(),
                 clasificador_1nn_con_pesos(&datos,
                                            &datos,
                                            &pesos_aux).2
                )
            );
            
            mut_realizadas += 1;
        }
        
        contador_evaluaciones += mutaciones_esperadas;

        // Remplazo y elitismo

        // Buscamos si la mejor solución está en la población
        // provisional
        let f_mejor_cromosoma_gen_anterior =
            clasificador_1nn_con_pesos(&datos,
                                       &datos,
                                       &poblacion[mejor_cromosoma]).2;
        // println!("Eval: {} Mejor padre fitness: {}, ",
        //          contador_evaluaciones,
        //          f_mejor_cromosoma_gen_anterior); 

        let mut mejor_crom_introducido = false;
        let mut peor_cromosoma = 0;
        let mut peor_f = pob_provisional[0].1;
        let mut mejor_cromosoma_actual = 0;
        let mut mejor_f = pob_provisional[0].1;
        
        for i in 0..pob_provisional.len() {
            let f_actual = pob_provisional[i].1;
            if f_actual == f_mejor_cromosoma_gen_anterior {
                mejor_crom_introducido = true;
            }

            if f_actual < peor_f {
                peor_cromosoma = i;
                peor_f = f_actual;
            }
            if f_actual > mejor_f {
                mejor_cromosoma_actual = i;
                mejor_f = f_actual;
            }
        }

        
        if f_mejor_cromosoma_gen_anterior < mejor_f {
            pesos = pob_provisional[mejor_cromosoma_actual].0.clone();
        } else {
            pesos = poblacion[mejor_cromosoma].clone();
        }

        // println!("{}\t{}", generacion, mejor_f);
        // generacion += 1;
        
        if !mejor_crom_introducido {
            pob_provisional.remove(peor_cromosoma);
            pob_provisional.push(
                (poblacion[mejor_cromosoma].clone(),
                 f_mejor_cromosoma_gen_anterior)
            );   
        }
        
        
        
        poblacion = Vec::with_capacity(TAM_POBLACION_GEN);
        for cromosoma in pob_provisional.iter_mut() {
            poblacion.push(cromosoma.0.clone());
        }

    }

    
    return pesos;
}

fn alg_genetico_estacionario<T: DataElem<T> + Copy + Clone>(
    datos: &Vec<T>,
    seed_u64: u64,
    variante_cruce: VarianteCruce)
    -> Vec<f32> {

    let num_attributes = T::get_num_attributes();
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed_u64); // Para generar números aleatorios con una semilla
    let mut pesos: Vec<f32> = vec![0.0; num_attributes];

    // Distribución para las mutaciones
    let distribucion_normal = Normal::new(0.0, VARIANZA_MUTACIONES); 

    
    // Generar población inicial
    let mut poblacion: Vec<Vec<f32>> =
        Vec::with_capacity(TAM_POBLACION_GEN);

    inicializar_poblacion(&mut poblacion, &mut rng, num_attributes,
    TAM_POBLACION_GEN);
    
    // Condición de parada: 15000 evaluaciones
    let mut contador_evaluaciones: usize = 0;

    // Evaluamos esta población
    let mut pob_evaluada: Vec<(Vec<f32>, f32)> =
        Vec::with_capacity(TAM_POBLACION_GEN);
    
    for i in 0..poblacion.len() {
        pob_evaluada.push(
            (
                poblacion[i].clone(),
                clasificador_1nn_con_pesos(&datos,
                                           &datos,
                                           &poblacion[i]).2
            )
        );
    }
    
    contador_evaluaciones += TAM_POBLACION_GEN;

    let mut generacion = 0;
    while contador_evaluaciones < MAXIMO_EVALUACIONES_F_OBJ {
        
        // Seleccionamos los dos padres para el estacionario
        let mut seleccionados: Vec<(Vec<f32>, f32)> =
            Vec::with_capacity(PADRES_ESTACIONARIO);
        
        while seleccionados.len() < PADRES_ESTACIONARIO {
            let candidato1 = rng.gen_range(0, TAM_POBLACION_GEN);
            let candidato2 = rng.gen_range(0, TAM_POBLACION_GEN);

            let ganador = torneo_binario(candidato1,
                                         candidato2,
                                         &pob_evaluada);
            seleccionados.push(pob_evaluada[ganador].clone());
        }

        let mut pob_provisional: Vec<(Vec<f32>, f32)> =
            Vec::with_capacity(2);

        
        // Ahora cruzamos los padres hasta obtener los dos candidatos
        // a entrar en la población de la siguiente generación
        let n_cruces: usize = 2;
        
        if variante_cruce == VarianteCruce::ARIT {
            cruce_aritmetico(&datos, &seleccionados, &mut pob_provisional, n_cruces, num_attributes);
        } else {
            cruce_blx(&datos,
                      &seleccionados,
                      &mut pob_provisional,
                      n_cruces,
                      num_attributes,
                      &mut rng);
        }

        // Mutamos el número de genes esperado
        let mutaciones_esperadas = (num_attributes as f32 *
                                    TAM_POBLACION_GEN as f32 *
                                    PROB_MUTACION
        ).trunc() as usize;


        let mut mut_realizadas = 0;
        while mut_realizadas < mutaciones_esperadas {
            let cromosoma_mut = rng.gen_range(0, seleccionados.len());
            let gen_mut = rng.gen_range(0, num_attributes);

            let mut pesos_aux =
                pob_provisional[cromosoma_mut].0.clone(); 
            
            pesos_aux[gen_mut] +=
                distribucion_normal.sample(&mut rng) as f32;
            if pesos_aux[gen_mut] < 0.0 {
                pesos_aux[gen_mut] = 0.0;
            } else if pesos_aux[gen_mut] > 1.0 {
                pesos_aux[gen_mut] = 1.0;
            }

            pob_provisional.remove(cromosoma_mut);
            pob_provisional.push(
                (pesos_aux.clone(),
                 clasificador_1nn_con_pesos(&datos,
                                            &datos,
                                            &pesos_aux).2
                )
            );            
            mut_realizadas += 1;
            contador_evaluaciones += 1;
        }

        // Reemplazamos los dos peores cromosomas de la generación
        // anterior por estos
        // (En realidad sustituimos cada hijo por el peor que haya en
        // ese instante, siendo posible que el peor sea el hijo que
        // acabamos de introducir)
        
        //let mut mejor_f = 0.0; // DEBUG: Mostrar mejor por generacion
        for i in 0..seleccionados.len() {
            let mut min_f = 100.0;
            let mut min_pos = 0;
            let mut counter = 0;
            // Buscamos el peor cromosoma
            for cromosoma in pob_evaluada.iter() {
                if cromosoma.1 < min_f {
                    min_f = cromosoma.1;
                    min_pos = counter;
                }

                // // DEBUG: Mostrar mejor por generación
                // if cromosoma.1 > mejor_f {
                //     mejor_f = cromosoma.1;
                // }
                 counter += 1;
            }

            // Si es peor que uno de los candidatos, lo expulsamos
            if min_f < pob_provisional[i].1 {
                poblacion.remove(min_pos);
                poblacion.push(pob_provisional[i].0.clone());
                pob_evaluada.remove(min_pos);
                pob_evaluada.push(
                    (
                        pob_provisional[i].0.clone(),
                        pob_provisional[i].1)
                );
            }
        }

        // DEBUG: Mostrar mejor por generación///////////
        
        // println!("{}\t{}", generacion, mejor_f);
        // generacion += 1;
        
        ////////////////////
        
    }
        

    // Ahora buscamos el mejor cromosoma en la población final para
    // devolverlo como solución
    let mut mejor_cromosoma = 0;
    let mut pob_evaluada: Vec<(Vec<f32>, f32)> =
        Vec::with_capacity(TAM_POBLACION_GEN); 
        pob_evaluada.push(
            (
                poblacion[0].clone(),
                clasificador_1nn_con_pesos(&datos,
                                           &datos,
                                           &poblacion[0]).2
            )
        );
        for i in 0..poblacion.len() {
            if i != mejor_cromosoma {
                pob_evaluada.push(
                    (
                        poblacion[i].clone(),
                        clasificador_1nn_con_pesos(&datos,
                                                   &datos,
                                                   &poblacion[i]).2
                    )
                );

                if pob_evaluada[i].1 > pob_evaluada[mejor_cromosoma].1
                {
                    mejor_cromosoma = i;
                }        
            }
        }

    pesos = pob_evaluada[mejor_cromosoma].0.clone();
    
    return pesos;
}


// El algoritmo memético a desarrollar utilizará una búsqueda local
// sobre el algoritmo genético estacionario con cruce BLX, que ha sido
// el que mejor resultados nos ha dado 
fn alg_memetico<T: DataElem<T> + Copy + Clone>(
    datos: &Vec<T>,
    seed_u64: u64,
    variante_memetico: VarianteMemetico)
    -> Vec<f32> {
    let num_attributes = T::get_num_attributes();
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed_u64); // Para generar números aleatorios con una semilla
    let mut pesos: Vec<f32> = vec![0.0; num_attributes];

    // Distribución para las mutaciones
    let distribucion_normal = Normal::new(0.0, VARIANZA_MUTACIONES); 
        
    // Generar población inicial
    let mut poblacion: Vec<Vec<f32>> =
        Vec::with_capacity(TAM_POBLACION_MEM);

    inicializar_poblacion(&mut poblacion, &mut rng, num_attributes,
    TAM_POBLACION_MEM); 

    // Condición de parada: 15000 evaluaciones de f. obj.
    let mut contador_evaluaciones: usize = 0;
    let mut contador_busqueda_local = 0; // Cada diez ha de dispararse
    //    let mut generacion = 0; // DEBUG: Mejor por generación
    while contador_evaluaciones < MAXIMO_EVALUACIONES_F_OBJ {
        // Algoritmo genético generacional elitista con cruce BLX
        let mut pob_evaluada: Vec<(Vec<f32>, f32)> =
            Vec::with_capacity(TAM_POBLACION_MEM);
        let mut mejor_cromosoma = 0;
        pob_evaluada.push(
            (
                poblacion[0].clone(),
                clasificador_1nn_con_pesos(&datos,
                                           &datos,
                                           &poblacion[0]).2
            )
        );
        for i in 0..poblacion.len() {
            if i != mejor_cromosoma {
                pob_evaluada.push(
                    (
                        poblacion[i].clone(),
                        clasificador_1nn_con_pesos(&datos,
                                                   &datos,
                                                   &poblacion[i]).2
                    )
                );

                if pob_evaluada[i].1 > pob_evaluada[mejor_cromosoma].1
                {
                    mejor_cromosoma = i;
                }        
            }
        }
        
        contador_evaluaciones += TAM_POBLACION_MEM;

        //// Seleccionamos padres mediante torneo binario
        
        let mut seleccionados: Vec<(Vec<f32>, f32)> =
            Vec::with_capacity(TAM_POBLACION_MEM);

        let tam_poblacion_padres = TAM_POBLACION_MEM * 2;
        while seleccionados.len() < tam_poblacion_padres {
            let candidato1 = rng.gen_range(0, TAM_POBLACION_MEM);
            let candidato2 = rng.gen_range(0, TAM_POBLACION_MEM);

            let ganador = torneo_binario(candidato1,
                                        candidato2,
                                        &pob_evaluada);
            seleccionados.push(pob_evaluada[ganador].clone());
        }

        // Ahora aplicamos el cruce BLX
       
        let mut pob_provisional: Vec<(Vec<f32>, f32)> =
            Vec::with_capacity(TAM_POBLACION_MEM);

        let n_cruces: usize = ((tam_poblacion_padres as f32) *
                                   PROB_CRUCE_AGG).trunc() as usize;
            
        cruce_blx(&datos,
                  &seleccionados,
                  &mut pob_provisional,
                  n_cruces,
                  num_attributes,
                  &mut rng);

        contador_evaluaciones += n_cruces;
        

        // Completamos la población con padres de la anterior
        // generación
        let mut counter = n_cruces; // Introducimos desde el
        // último candidato a reproducirse hasta que se llene la
        // población
        while pob_provisional.len() < TAM_POBLACION_MEM {
            pob_provisional.push(
                (seleccionados[counter].0.clone(),
                 seleccionados[counter].1
                )
            );
            counter += 1;
        }

        // Mutamos el número de genes esperado
        let mutaciones_esperadas = (num_attributes as f32 *
                                    TAM_POBLACION_MEM as f32 *
                                    PROB_MUTACION
        ).trunc() as usize;

        let mut mut_realizadas = 0;
        
        while mut_realizadas < mutaciones_esperadas {
            let cromosoma_mut = rng.gen_range(0, TAM_POBLACION_MEM);
            let gen_mut = rng.gen_range(0, num_attributes);

            let mut pesos_aux =
            pob_provisional[cromosoma_mut].0.clone(); 
            
            pesos_aux[gen_mut] +=
                distribucion_normal.sample(&mut rng) as f32;
            if pesos_aux[gen_mut] < 0.0 {
                pesos_aux[gen_mut] = 0.0;
            } else if pesos_aux[gen_mut] > 1.0 {
                pesos_aux[gen_mut] = 1.0;
            }

            pob_provisional.remove(cromosoma_mut);
            pob_provisional.push(
                (pesos_aux.clone(),
                 clasificador_1nn_con_pesos(&datos,
                                            &datos,
                                            &pesos_aux).2
                )
            );
            mut_realizadas += 1;
        }
        
        contador_evaluaciones += mutaciones_esperadas;
        

        // Búsqueda local sobre la población existente
        contador_busqueda_local += 1;

        if contador_busqueda_local >= 10 {
            //println!("Lanzamos búsqueda local. Evaluaciones: {}", contador_evaluaciones);
            contador_busqueda_local = 0;

            if variante_memetico == VarianteMemetico::TODOS_CROMOSOMAS {
                // Lanzamos búsqueda local sobre todos los individuos
                for i in 0..poblacion.len() {
                   // println!("Candidato a mejorar {}",
                   //          pob_evaluada[i].1); 
                    let mut cont_eval_bl = 0;
                    let evaluaciones_bl = 2 * num_attributes;
                    while cont_eval_bl < evaluaciones_bl {
                        pob_evaluada[i].0 =
                            aux_busqueda_local(
                                &datos,
                                &mut rng,
                                &pob_evaluada[i].0,
                                pob_evaluada[i].1,
                                &distribucion_normal).clone();
                        pob_evaluada[i].1 =
                            clasificador_1nn_con_pesos(
                                &datos,
                                &datos,
                                &pob_evaluada[i].0).2; 
                        contador_evaluaciones += 2; // Evaluamos
                        // dentro de la búsqueda local y ahora también
                        cont_eval_bl += 1;
                    }
                    poblacion[i] = pob_evaluada[i].0.clone();
                }
            } else if variante_memetico == VarianteMemetico::CROM_ALEATORIO_P0_1 {
                let cromosomas_a_mutar = ((poblacion.len() as f32) *
                                         PROB_BUSQUEDA_MEMETICO).trunc() as usize;
                for _i in 0..cromosomas_a_mutar {
                    let indice_cromosoma =
                        rng.gen_range(0, poblacion.len());
                    let mut cont_eval_bl = 0;
                    let evaluaciones_bl = 2 * num_attributes;
                    while cont_eval_bl < evaluaciones_bl {
                        pob_evaluada[indice_cromosoma].0 =
                            aux_busqueda_local(
                                &datos,
                                &mut rng,
                                &pob_evaluada[indice_cromosoma].0,
                                pob_evaluada[indice_cromosoma].1,
                                &distribucion_normal).clone();
                        pob_evaluada[indice_cromosoma].1 =
                            clasificador_1nn_con_pesos(
                                &datos,
                                &datos,
                                &pob_evaluada[indice_cromosoma].0).2; 
                        contador_evaluaciones += 2; // Evaluamos
                        // dentro de la búsqueda local y ahora también
                        cont_eval_bl += 1;
                    }
                    poblacion[indice_cromosoma] = pob_evaluada[indice_cromosoma].0.clone();
                }
            } else {
                // Aplicamos la búsqueda sólo sobre el mejor cromosoma
                let indice_cromosoma = mejor_cromosoma;
                let mut cont_eval_bl = 0;
                let evaluaciones_bl = 2 * num_attributes;
                while cont_eval_bl < evaluaciones_bl {
                    pob_evaluada[indice_cromosoma].0 =
                        aux_busqueda_local(
                            &datos,
                            &mut rng,
                            &pob_evaluada[indice_cromosoma].0,
                            pob_evaluada[indice_cromosoma].1,
                            &distribucion_normal).clone();
                    pob_evaluada[indice_cromosoma].1 =
                        clasificador_1nn_con_pesos(
                            &datos,
                            &datos,
                            &pob_evaluada[indice_cromosoma].0).2; 
                    contador_evaluaciones += 2; // Evaluamos dentro de la búsqueda local y ahora también
                                                // Podríamos mejorar esto calculando solo una vez el f_obj
                    cont_eval_bl += 1;
                }
                poblacion[indice_cromosoma] = pob_evaluada[indice_cromosoma].0.clone();
            }
            //            println!("Finaliza BL");
        }

        // Remplazo y elitismo

        // Buscamos si la mejor solución está en la población
        // provisional
        let f_mejor_cromosoma_gen_anterior =
            clasificador_1nn_con_pesos(&datos,
                                       &datos,
                                       &poblacion[mejor_cromosoma]).2;

        let mut mejor_crom_introducido = false;
        let mut peor_cromosoma = 0;
        let mut peor_f = pob_provisional[0].1;
        let mut mejor_cromosoma_actual = 0;
        let mut mejor_f = pob_provisional[0].1;
        
        for i in 0..pob_provisional.len() {
            let f_actual = pob_provisional[i].1;
            if f_actual == f_mejor_cromosoma_gen_anterior {
                mejor_crom_introducido = true;
            }

            if f_actual < peor_f {
                peor_cromosoma = i;
                peor_f = f_actual;
            }
            if f_actual > mejor_f {
                mejor_cromosoma_actual = i;
                mejor_f = f_actual;
            }
        }

        

        
        if f_mejor_cromosoma_gen_anterior < mejor_f {
            pesos = pob_provisional[mejor_cromosoma_actual].0.clone();
            //println!("{}\t{}", generacion, mejor_f);// DEBUG: mejor por gen
        } else {
            pesos = poblacion[mejor_cromosoma].clone();
            //println!("{}\t{}", generacion, f_mejor_cromosoma_gen_anterior);
        }
        //generacion += 1;
        
        
        if !mejor_crom_introducido {
            pob_provisional.remove(peor_cromosoma);
            pob_provisional.push(
                (poblacion[mejor_cromosoma].clone(),
                 f_mejor_cromosoma_gen_anterior)
            );   
        }
        
        poblacion = Vec::with_capacity(TAM_POBLACION_MEM);
        for cromosoma in pob_provisional.iter_mut() {
            poblacion.push(cromosoma.0.clone());
        }
    }
    process::exit(1);
    return pesos;
}


//////////////////////////////////////////////////
////////// PROCEDIMIENTOS GENERALES //////////////
//////////////////////////////////////////////////

// Crea particiones con distribución de clases uniforme

fn crear_particiones<T: DataElem<T> + Copy + Clone>(
    datos: &Vec<T>)
    -> Vec<Vec<T>> {
    
    let mut particiones: Vec<Vec<T>> = Vec::new();
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

fn normalizar_datos<T: DataElem<T> + Copy + Clone>(
    datos: &mut Vec<T>) {
    
    // Calculamos el máximo y el mínimo para cada atributo  y lo
    // almacenamos en un vector de máximos/mínimos
    let num_attributes = T::get_num_attributes();
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
            if maximos[atributo] - minimos[atributo] != 0.0 {
                miembro.set_attribute(atributo, (miembro.get_attribute(atributo) - minimos[atributo]) / (maximos[atributo] - minimos[atributo]));
            }
        }
    }
}
//////////////////////////////////////////////////
///// Procedimientos generales genéticos /////////
//////////////////////////////////////////////////


fn inicializar_poblacion(
    poblacion: &mut Vec<Vec<f32>>,
    mut rng: &mut StdRng,
    num_attributes: usize,
    tam_poblacion: usize) {
// Distribución para la inicialización
let distribucion_uniforme = Uniform::new(0.0, 1.0);

 // Inicializamos los cromosomas
    for i in 0..tam_poblacion {
        poblacion.push(Vec::with_capacity(num_attributes));
        for _ in 0..num_attributes {
            poblacion[i].push(distribucion_uniforme.sample(&mut rng));            
        }
    }
}

fn cruce_aritmetico<T: DataElem<T> + Copy + Clone>(
    datos: &Vec<T>,
    seleccionados: & Vec<(Vec<f32>, f32)>,
    pob_provisional: &mut Vec<(Vec<f32>, f32)>,
    n_cruces: usize,
    num_attributes: usize) {
    
    let mut i = 0;
    while i < n_cruces {
        let mut cromosoma = Vec::with_capacity(num_attributes);
        for j in 0..num_attributes {
            let gen;
            let index = if seleccionados.len() > 2 {i} else {0};
            // Distinguimos estacionario
            // de generacional cuando hay más de dos padres 
            gen = (seleccionados[index].0[j] +
                   seleccionados[index+1].0[j]) / 2.0;
            
            cromosoma.push(gen);
        }

        // En el algoritmo generacional con elitismo necesitamos saber
        // cuál es el peor para sustituirlo en caso de necesidad. En
        // el estacionario solo calculamos la f_obj al inicio
        //
        // Además el índice se incrementa en 2 porque se reutiliza
        // para indexar los padres a cruzar
        if seleccionados.len() > 2 {
            pob_provisional.push(
                (cromosoma.clone(),
                 clasificador_1nn_con_pesos(&datos,
                                            &datos,
                                            &cromosoma).2
                )
            );
            
            i += 2;
        } else {
            pob_provisional.push(
                (cromosoma.clone(),
                 0.0
                )
            );
            i += 1;
        }
    }
}


fn cruce_blx<T: DataElem<T> + Copy + Clone>(
    datos: &Vec<T>,
    seleccionados: & Vec<(Vec<f32>, f32)>,
    pob_provisional: &mut Vec<(Vec<f32>, f32)>,
    n_cruces: usize,
    num_attributes: usize,
    mut rng: &mut StdRng) {
    
    let mut i = 0;
    while i < n_cruces {
        let mut cromosoma =
            Vec::with_capacity(num_attributes);
        
        let index = if seleccionados.len() > 2 {i} else
        {0}; // Así distinguimos  entre AGE y AGG para acceder
        // a los padres
        
        for j in 0..num_attributes {
            let cmax;
            let cmin;

            
            if seleccionados[index].0[j] > seleccionados[index+1].0[j] {
                cmax = seleccionados[index].0[j];
                cmin = seleccionados[index+1].0[j];
            } else {
                cmin = seleccionados[index].0[j];
                cmax = seleccionados[index+1].0[j];
            } 
            let interval = (cmax - cmin) * BLX_VALUE as f32;

            let mut gen;
            if cmax != cmin {
                gen = rng.gen_range(cmin - interval, cmax +
                                    interval);
                if gen < 0.0 {
                    gen = 0.0;
                } else if gen > 1.0 {
                    gen = 0.0;
                }
                
            } else {
                gen = seleccionados[index].0[j];
            }
            cromosoma.push(gen);
        }

        pob_provisional.push(
            (cromosoma.clone(),
             clasificador_1nn_con_pesos(&datos,
                                        &datos,
                                        &cromosoma).2
            )
        );
        
        if seleccionados.len() > 2 {
            i += 2;
        } else {
            i += 1;
        }
}            
}
    
// Torneo binario
//
// Recibe los datos de entrada y dos candidatos y decide el ganador
// del torneo binario.
//
// Devuelve el índice del ganador

fn torneo_binario(
    candidato1: usize,
    candidato2: usize,
    pob_evaluada: &Vec<(Vec<f32>,f32)>)
    -> usize {
    
    let f_1 = pob_evaluada[candidato1].1;
    let f_2 = pob_evaluada[candidato2].1;

//    println!("{}  {}", f_1, f_2);
    
    return if f_1 >= f_2 {candidato1} else {candidato2};
}

fn aux_busqueda_local<T: DataElem<T> + Copy + Clone>(
    datos: &Vec<T>,
    mut rng: &mut StdRng,
    pesos: &Vec<f32>,
    mejor_f_obj: f32,
    distribucion_normal: &Normal )
    -> Vec<f32> {
    let mut pesos_aux = pesos.clone();
    let num_attributes = T::get_num_attributes();
    
    
    let indice_a_mejorar = rng.gen_range(0, num_attributes);

    pesos_aux[indice_a_mejorar] +=
        distribucion_normal.sample(&mut rng) as f32;
    if pesos_aux[indice_a_mejorar] < 0.0 {
        pesos_aux[indice_a_mejorar] = 0.0;
    } else if pesos_aux[indice_a_mejorar] > 1.0 {
        pesos_aux[indice_a_mejorar] = 1.0;
    }

    let f_obj_actual = clasificador_1nn_con_pesos(
        &datos,
        &datos,
        &pesos_aux).2;
    //println!("Mutación bl f={}", f_obj_actual);

    // Si se mejora se devuelve el peso mejorado, en caso contrario se
    // devuelve el que ya existía
    if f_obj_actual > mejor_f_obj {

        //  println!("Vector de pesos mejorado. F_obj: {}",
        //         f_obj_actual);
        return pesos_aux;
    } else {
        return pesos.clone();
    }
}



// Método principal: Ejecuta el código de la práctica

fn execute<T: DataElem<T> + Copy + Clone>(
    path: &str,
    seed_u64: u64)
    -> Result<(), Box<Error>> {

    // Reads data, then works with it
    let mut data: Vec<T> = Vec::new();
    let mut rdr = csv::Reader::from_path(&path)?;

    let mut current_id = 0;
    for result in rdr.records() {
        let mut aux_record = T::new();
        let record = result?;

        let mut counter = 0;

        aux_record.set_id(current_id);

        for field in record.iter() {
            if counter != T::get_num_attributes() {
                aux_record.set_attribute(counter, field.parse::<f32>().unwrap());
            } else {
                aux_record.set_class(field.parse::<i32>().unwrap());
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
        let mut conjunto_entrenamiento: Vec<T> =
            Vec::new();
        let mut conjunto_validacion: Vec<T> = Vec::new();

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

        // Resultados
        println!("-----------------------------------------");
        println!("Resultados partición: {} ", n_ejecucion);
        
        // Muestra resultados 1nn
  /*      
        println!("-- Resultados clasificador 1nn");
        println!("\tT_clas\tT_red\tT_obj\tTiempo");
        println!("\t{}\t{}\t{}\t{}ms\n", resultados_1nn.0, resultados_1nn.1, resultados_1nn.2, tiempo_total);

        tiempo_inicial = Instant::now();
        
        let pesos_relief = algoritmo_relief(&conjunto_entrenamiento);
        let resultados_relief =
        clasificador_1nn_con_pesos(&conjunto_entrenamiento,
                                   &conjunto_validacion, &pesos_relief);

        tiempo_total = tiempo_inicial.elapsed().as_millis();

        // Muestra resultados relief
        
        println!("-- Resultados clasificador RELIEF");
        println!("\tT_clas\tT_red\tT_obj\tTiempo");
        println!("\t{}\t{}\t{}\t{}ms\n", resultados_relief.0, resultados_relief.1, resultados_relief.2, tiempo_total);
        
        // Búsqueda local

        tiempo_inicial = Instant::now();
        
        let pesos_busqueda_local =
            busqueda_local(&conjunto_entrenamiento, seed_u64);

        let resultados_bl =
        clasificador_1nn_con_pesos(&conjunto_entrenamiento,
                                   &conjunto_validacion,
                                   &pesos_busqueda_local); 

        tiempo_total = tiempo_inicial.elapsed().as_millis();

        println!("-- Resultados clasificador búsqueda local");
        println!("\tT_clas\tT_red\tT_obj\tTiempo");
        println!("\t{}\t{}\t{}\t{}ms\n", resultados_bl.0, resultados_bl.1, resultados_bl.2, tiempo_total);
         */
        
        tiempo_inicial = Instant::now();
        
        let mut variante_cruce = VarianteCruce::ARIT;
        
        let pesos_agg = alg_genetico_generacional_elitista(&conjunto_entrenamiento,
        seed_u64, variante_cruce);
        let resultados_agg =
        clasificador_1nn_con_pesos(&conjunto_entrenamiento,
        &conjunto_validacion, &pesos_agg); 

        tiempo_total = tiempo_inicial.elapsed().as_millis();


        println!("-- Resultados algoritmo genético generacional. Cruce aritmético. Elitista.");
        println!("\tT_clas\tT_red\tT_obj\tTiempo");
        println!("\t{}\t{}\t{}\t{}ms\n", resultados_agg.0, resultados_agg.1, resultados_agg.2, tiempo_total);
        

        tiempo_inicial = Instant::now();
        
        variante_cruce = VarianteCruce::BLX;
        let pesos_agg_blx = alg_genetico_generacional_elitista(&conjunto_entrenamiento,
        seed_u64, variante_cruce);
        let resultados_agg_blx =
        clasificador_1nn_con_pesos(&conjunto_entrenamiento,
        &conjunto_validacion, &pesos_agg_blx); 

        tiempo_total = tiempo_inicial.elapsed().as_millis();


        println!("-- Resultados algoritmo genético generacional. Cruce BLX. Elitista.");
        println!("\tT_clas\tT_red\tT_obj\tTiempo");
        println!("\t{}\t{}\t{}\t{}ms\n", resultados_agg_blx.0, resultados_agg_blx.1, resultados_agg_blx.2, tiempo_total);

        tiempo_inicial = Instant::now();
        

        variante_cruce = VarianteCruce::ARIT;
        let pesos_age =
            alg_genetico_estacionario(
                &conjunto_entrenamiento,
                seed_u64,
                variante_cruce); 
        let resultados_age =
            clasificador_1nn_con_pesos(&conjunto_entrenamiento,
                                       &conjunto_validacion, &pesos_age); 

        tiempo_total = tiempo_inicial.elapsed().as_millis();

        println!("-- Resultados algoritmo genético estacionario. Cruce aritmético.");
        println!("\tT_clas\tT_red\tT_obj\tTiempo");
        println!("\t{}\t{}\t{}\t{}ms\n", resultados_age.0, resultados_age.1, resultados_age.2, tiempo_total);
        
        
        tiempo_inicial = Instant::now();
        
        variante_cruce = VarianteCruce::BLX;
        let pesos_age_blx =
            alg_genetico_estacionario(
                &conjunto_entrenamiento,
                seed_u64,
                variante_cruce); 
        let resultados_age_blx =
            clasificador_1nn_con_pesos(&conjunto_entrenamiento,
                                       &conjunto_validacion, &pesos_age_blx); 

        tiempo_total = tiempo_inicial.elapsed().as_millis();


        println!("-- Resultados algoritmo genético estacionario. Cruce BLX.");
        println!("\tT_clas\tT_red\tT_obj\tTiempo");
        println!("\t{}\t{}\t{}\t{}ms\n", resultados_age_blx.0, resultados_age_blx.1, resultados_age_blx.2, tiempo_total);
 
        tiempo_inicial = Instant::now();
        
 
        
        let mut variante_memetico =
            VarianteMemetico::TODOS_CROMOSOMAS;
        
        let pesos_mem_bl_todos =
            alg_memetico(
                &conjunto_entrenamiento,
                seed_u64,
                variante_memetico); 
        let resultados_mem_bl_todos =
            clasificador_1nn_con_pesos(&conjunto_entrenamiento,
                                       &conjunto_validacion,
                                       &pesos_mem_bl_todos); 

        tiempo_total = tiempo_inicial.elapsed().as_millis();


        println!("-- Resultados algoritmo memético con BL sobre todos los cromosomas. Cruce BLX.");
        println!("\tT_clas\tT_red\tT_obj\tTiempo");
        println!("\t{}\t{}\t{}\t{}ms\n", resultados_mem_bl_todos.0, resultados_mem_bl_todos.1, resultados_mem_bl_todos.2, tiempo_total);

        tiempo_inicial = Instant::now();

        variante_memetico = VarianteMemetico::CROM_ALEATORIO_P0_1;
        let pesos_mem_bl_aleatorio =
            alg_memetico(
                &conjunto_entrenamiento,
                seed_u64,
                variante_memetico); 
        let resultados_mem_bl_aleatorio =
            clasificador_1nn_con_pesos(&conjunto_entrenamiento,
                                       &conjunto_validacion,
                                       &pesos_mem_bl_aleatorio); 

        tiempo_total = tiempo_inicial.elapsed().as_millis();


        println!("-- Resultados algoritmo memético con BL sobre cromosomas aleatorios con p=0.1. Cruce BLX.");
        println!("\tT_clas\tT_red\tT_obj\tTiempo");
        println!("\t{}\t{}\t{}\t{}ms\n", resultados_mem_bl_aleatorio.0, resultados_mem_bl_aleatorio.1, resultados_mem_bl_aleatorio.2, tiempo_total);


        tiempo_inicial = Instant::now();
         
        variante_memetico = VarianteMemetico::MEJORES_N_CROMOSOMAS;

                let pesos_mem_bl_mejores =
            alg_memetico(
                &conjunto_entrenamiento,
                seed_u64,
                variante_memetico); 
        let resultados_mem_bl_mejores =
            clasificador_1nn_con_pesos(&conjunto_entrenamiento,
                                       &conjunto_validacion,
                                       &pesos_mem_bl_mejores); 

        tiempo_total = tiempo_inicial.elapsed().as_millis();


        println!("-- Resultados algoritmo memético con BL sobre los mejores n cromosomas. Cruce BLX.");
        println!("\tT_clas\tT_red\tT_obj\tTiempo");
        println!("\t{}\t{}\t{}\t{}ms\n", resultados_mem_bl_mejores.0, resultados_mem_bl_mejores.1, resultados_mem_bl_mejores.2, tiempo_total);

        
    }
    
    
    Ok(())
}

fn main() {

    let args: Vec<_> = env::args().collect();
    let mut seed_u64: u64 = 4;
    
    if args.len() == 2 {
        seed_u64 = args[1].parse::<u64>().unwrap();
        println!("Se usará como semilla: {}", seed_u64);
    } else if args.len() != 2 {
        println!("* Formato de introducción de semilla incorrecto, se usará 4 como semilla.\nPara usar una semilla concreta utilice cargo run --release <semilla>");
    }
    println!("-----------------------------------------");
    println!("Análisis para el archivo: colposcopy");
    if let Err(err) = execute::<ColposcopyRecord>("../data/colposcopy.csv", seed_u64) {
        println!("error: {}", err);
        process::exit(1);
    }
       
    println!("-----------------------------------------");
    println!("Análisis para el archivo: ionosphere");
    if let Err(err) = execute::<IonosphereRecord>("../data/ionosphere.csv", seed_u64) {
        println!("error: {}", err);
        process::exit(1);
    }
      
    println!("-----------------------------------------");
    println!("Análisis para el archivo: texture");
    if let Err(err) = execute::<TextureRecord>("../data/texture.csv", seed_u64) {
        println!("error: {}", err);
        process::exit(1);
    }


}
