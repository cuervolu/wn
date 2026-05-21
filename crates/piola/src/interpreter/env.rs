use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{error::RuntimeError, value::Valor};

#[derive(Debug, Clone)]
pub struct Entorno {
    valores: HashMap<String, (Valor, bool)>, // (value, is_duro)
    padre: Option<Rc<RefCell<Entorno>>>,
}

impl Entorno {
    pub fn nuevo() -> Self {
        Entorno {
            valores: HashMap::new(),
            padre: None,
        }
    }

    pub fn nuevo_hijo(padre: Rc<RefCell<Entorno>>) -> Self {
        Entorno {
            valores: HashMap::new(),
            padre: Some(padre),
        }
    }

    pub fn definir(
        &mut self,
        nombre: &str,
        valor: Valor,
        es_duro: bool,
    ) -> Result<(), RuntimeError> {
        if let Some((_, true)) = self.valores.get(nombre) {
            return Err(RuntimeError::ConstanteInmutable(nombre.to_string()));
        }
        self.valores.insert(nombre.to_string(), (valor, es_duro));
        Ok(())
    }

    pub fn asignar(&mut self, nombre: &str, valor: Valor) -> Result<(), RuntimeError> {
        if let Some((entry, es_duro)) = self.valores.get_mut(nombre) {
            if *es_duro {
                return Err(RuntimeError::ConstanteInmutable(nombre.to_string()));
            }
            *entry = valor;
            return Ok(());
        }
        if let Some(padre) = &self.padre {
            return padre.borrow_mut().asignar(nombre, valor);
        }
        Err(RuntimeError::VarNoDefinida(nombre.to_string()))
    }

    pub fn obtener(&self, nombre: &str) -> Option<Valor> {
        if let Some((val, _)) = self.valores.get(nombre) {
            return Some(val.clone());
        }
        if let Some(padre) = &self.padre {
            return padre.borrow().obtener(nombre);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    const PI_NUM: f64 = std::f64::consts::PI;

    #[test]
    fn duro_no_permite_redeclaracion_mismo_scope() {
        let mut env = Entorno::nuevo();
        env.definir("PI", Valor::Numero(PI_NUM), true).unwrap();

        let resultado = env.definir("PI", Valor::Numero(99.0), true);

        assert!(
            matches!(resultado, Err(RuntimeError::ConstanteInmutable(nombre)) if nombre == "PI")
        );
    }

    #[test]
    fn duro_permite_shadowing_en_scope_hijo() {
        let padre = Rc::new(RefCell::new(Entorno::nuevo()));
        padre
            .borrow_mut()
            .definir("PI", Valor::Numero(PI_NUM), true)
            .unwrap();

        let mut hijo = Entorno::nuevo_hijo(Rc::clone(&padre));

        let resultado = hijo.definir("PI", Valor::Numero(99.0), true);

        assert!(resultado.is_ok());
        assert_eq!(hijo.obtener("PI"), Some(Valor::Numero(99.0)));
    }

    #[test]
    fn wea_permite_redeclaracion_mismo_scope() {
        let mut env = Entorno::nuevo();
        env.definir("x", Valor::Numero(10.0), false).unwrap();

        let resultado = env.definir("x", Valor::Texto("hola".to_string()), false);

        assert!(resultado.is_ok());
        assert_eq!(env.obtener("x"), Some(Valor::Texto("hola".to_string())));
    }

    #[test]
    fn scope_hijo_puede_leer_variable_del_padre() {
        let padre = Rc::new(RefCell::new(Entorno::nuevo()));
        padre
            .borrow_mut()
            .definir("x", Valor::Numero(10.0), false)
            .unwrap();

        let hijo = Entorno::nuevo_hijo(Rc::clone(&padre));

        assert_eq!(hijo.obtener("x"), Some(Valor::Numero(10.0)));
    }

    #[test]
    fn scope_padre_no_puede_leer_variable_del_hijo() {
        let padre = Rc::new(RefCell::new(Entorno::nuevo()));
        let mut hijo = Entorno::nuevo_hijo(Rc::clone(&padre));
        hijo.definir("x", Valor::Numero(10.0), false).unwrap();

        assert_eq!(padre.borrow().obtener("x"), None);
    }
}
