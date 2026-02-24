// =============================================================================
// 🔥 FORGE — Motor Core: Grafo de Tareas (DAG)
// =============================================================================
// Grafo Acíclico Dirigido para ordenar y paralelizar tareas de build.
// Patrón moderno: diseño funcional con iteradores y detección de ciclos.
// =============================================================================

use std::collections::{HashMap, HashSet, VecDeque};

use crate::error::{ForgeError, ForgeResult};

/// Representa una tarea en el grafo de build.
#[derive(Debug, Clone)]
pub struct Task {
    /// Nombre único de la tarea
    pub name: String,

    /// Descripción legible de la tarea
    pub description: String,

    /// Nombres de tareas de las que depende
    pub depends_on: Vec<String>,

    /// Acción a ejecutar (comando externo o función interna)
    pub action: TaskAction,
}

/// Tipos de acción que puede ejecutar una tarea.
#[derive(Debug, Clone)]
pub enum TaskAction {
    /// Ejecutar un comando del sistema
    Command(String),

    /// Tarea interna del build system (compilar, empaquetar, etc.)
    Internal(InternalTask),

    /// Tarea compuesta (agrupa otras tareas)
    Composite,
}

/// Tareas internas predefinidas del build system.
#[derive(Debug, Clone)]
pub enum InternalTask {
    /// Compilar código fuente
    Compile,
    /// Ejecutar el programa
    Run,
    /// Ejecutar tests
    Test,
    /// Empaquetar (JAR, wheel, etc.)
    Package,
    /// Limpiar artefactos
    Clean,
    /// Resolver y descargar dependencias
    ResolveDeps,
}

/// Grafo Acíclico Dirigido (DAG) de tareas.
///
/// Permite:
/// - Agregar tareas con dependencias
/// - Detectar ciclos
/// - Obtener orden de ejecución (topológico)
/// - Identificar tareas que pueden ejecutarse en paralelo
#[derive(Debug, Default)]
pub struct TaskGraph {
    /// Mapa de nombre de tarea -> tarea
    tasks: HashMap<String, Task>,

    /// Grafo de adyacencia: nombre -> dependencias
    edges: HashMap<String, Vec<String>>,
}

impl TaskGraph {
    /// Crea un nuevo grafo vacío.
    pub fn new() -> Self {
        Self::default()
    }

    /// Agrega una tarea al grafo.
    pub fn add_task(&mut self, task: Task) -> ForgeResult<()> {
        let name = task.name.clone();
        let deps = task.depends_on.clone();

        self.tasks.insert(name.clone(), task);
        self.edges.insert(name, deps);

        Ok(())
    }

    /// Verifica que no existan ciclos en el grafo.
    pub fn validate(&self) -> ForgeResult<()> {
        // Algoritmo de detección de ciclos usando DFS con estados
        #[derive(PartialEq)]
        enum State {
            NotVisited,
            InProgress,
            Done,
        }

        let mut states: HashMap<&str, State> = HashMap::new();
        for name in self.tasks.keys() {
            states.insert(name.as_str(), State::NotVisited);
        }

        fn dfs<'a>(
            node: &'a str,
            edges: &'a HashMap<String, Vec<String>>,
            states: &mut HashMap<&'a str, State>,
            path: &mut Vec<&'a str>,
        ) -> Result<(), String> {
            states.insert(node, State::InProgress);
            path.push(node);

            if let Some(deps) = edges.get(node) {
                for dep in deps {
                    match states.get(dep.as_str()) {
                        Some(State::InProgress) => {
                            path.push(dep.as_str());
                            let cycle_start = path.iter().position(|&n| n == dep.as_str()).unwrap();
                            let cycle: Vec<&str> = path[cycle_start..].to_vec();
                            return Err(cycle.join(" → "));
                        }
                        Some(State::NotVisited) | None => {
                            dfs(dep.as_str(), edges, states, path)?;
                        }
                        Some(State::Done) => {}
                    }
                }
            }

            path.pop();
            states.insert(node, State::Done);
            Ok(())
        }

        let task_names: Vec<String> = self.tasks.keys().cloned().collect();
        for name in &task_names {
            if states.get(name.as_str()) == Some(&State::NotVisited) {
                let mut path = Vec::new();
                dfs(name.as_str(), &self.edges, &mut states, &mut path).map_err(|cycle| {
                    ForgeError::CyclicDependency { cycle }
                })?;
            }
        }

        // Verificar que todas las dependencias referenciadas existen
        for (task_name, deps) in &self.edges {
            for dep in deps {
                if !self.tasks.contains_key(dep) {
                    return Err(ForgeError::TaskNotFound {
                        task_name: format!(
                            "'{}' (referenciada por '{}')",
                            dep, task_name
                        ),
                    }
                    .into());
                }
            }
        }

        Ok(())
    }

    /// Devuelve las tareas en orden topológico (respetando dependencias).
    pub fn topological_order(&self) -> ForgeResult<Vec<String>> {
        self.validate()?;

        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for name in self.tasks.keys() {
            in_degree.insert(name.as_str(), 0);
        }

        // Calcular grado de entrada
        for deps in self.edges.values() {
            for dep in deps {
                if let Some(count) = in_degree.get_mut(dep.as_str()) {
                    *count += 1;
                }
            }
        }

        // Invertir: las tareas con 0 dependencias entrantes van primero
        // Pero en nuestro grafo, edges[A] = [B] significa "A depende de B",
        // así que B debe ejecutarse antes que A.

        // Recalcular: in_degree[B] cuenta cuántas tareas dependen de B
        // Lo que necesitamos es: ¿cuántas dependencias tiene cada tarea?
        let mut dep_count: HashMap<&str, usize> = HashMap::new();
        for (name, deps) in &self.edges {
            dep_count.insert(name.as_str(), deps.len());
        }
        for name in self.tasks.keys() {
            dep_count.entry(name.as_str()).or_insert(0);
        }

        let mut queue: VecDeque<&str> = VecDeque::new();
        for (name, &count) in &dep_count {
            if count == 0 {
                queue.push_back(name);
            }
        }

        let mut order = Vec::new();

        while let Some(current) = queue.pop_front() {
            order.push(current.to_string());

            // Para todas las tareas que dependen de `current`, reducir su conteo
            for (name, deps) in &self.edges {
                if deps.iter().any(|d| d.as_str() == current) {
                    if let Some(count) = dep_count.get_mut(name.as_str()) {
                        *count -= 1;
                        if *count == 0 {
                            queue.push_back(name.as_str());
                        }
                    }
                }
            }
        }

        Ok(order)
    }

    /// Devuelve los "niveles" de ejecución: tareas en el mismo nivel
    /// pueden ejecutarse en paralelo.
    pub fn parallel_levels(&self) -> ForgeResult<Vec<Vec<String>>> {
        self.validate()?;

        let mut dep_count: HashMap<String, usize> = HashMap::new();
        for (name, deps) in &self.edges {
            dep_count.insert(name.clone(), deps.len());
        }
        for name in self.tasks.keys() {
            dep_count.entry(name.clone()).or_insert(0);
        }

        let mut levels: Vec<Vec<String>> = Vec::new();
        let mut completed: HashSet<String> = HashSet::new();

        loop {
            // Encontrar todas las tareas con 0 dependencias pendientes
            let ready: Vec<String> = dep_count
                .iter()
                .filter(|(name, &count)| count == 0 && !completed.contains(name.as_str()))
                .map(|(name, _)| name.clone())
                .collect();

            if ready.is_empty() {
                break;
            }

            // Marcar como completadas
            for task in &ready {
                completed.insert(task.clone());
            }

            // Reducir dependencias de tareas que dependen de las completadas
            for (name, deps) in &self.edges {
                if !completed.contains(name) {
                    let resolved = deps.iter().filter(|d| completed.contains(d.as_str())).count();
                    dep_count.insert(name.clone(), deps.len() - resolved);
                }
            }

            levels.push(ready);
        }

        Ok(levels)
    }

    /// Devuelve una tarea por nombre.
    pub fn get_task(&self, name: &str) -> Option<&Task> {
        self.tasks.get(name)
    }

    /// Devuelve el número de tareas.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Verifica si el grafo está vacío.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(name: &str, deps: &[&str], action: TaskAction) -> Task {
        Task {
            name: name.to_string(),
            description: format!("Tarea: {}", name),
            depends_on: deps.iter().map(|s| s.to_string()).collect(),
            action,
        }
    }

    #[test]
    fn test_topological_order() {
        let mut graph = TaskGraph::new();
        graph
            .add_task(make_task("clean", &[], TaskAction::Internal(InternalTask::Clean)))
            .unwrap();
        graph
            .add_task(make_task("compile", &["clean"], TaskAction::Internal(InternalTask::Compile)))
            .unwrap();
        graph
            .add_task(make_task("test", &["compile"], TaskAction::Internal(InternalTask::Test)))
            .unwrap();
        graph
            .add_task(make_task("package", &["compile"], TaskAction::Internal(InternalTask::Package)))
            .unwrap();

        let order = graph.topological_order().unwrap();
        let clean_pos = order.iter().position(|t| t == "clean").unwrap();
        let compile_pos = order.iter().position(|t| t == "compile").unwrap();
        let test_pos = order.iter().position(|t| t == "test").unwrap();

        assert!(clean_pos < compile_pos);
        assert!(compile_pos < test_pos);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = TaskGraph::new();
        graph
            .add_task(make_task("a", &["b"], TaskAction::Composite))
            .unwrap();
        graph
            .add_task(make_task("b", &["c"], TaskAction::Composite))
            .unwrap();
        graph
            .add_task(make_task("c", &["a"], TaskAction::Composite))
            .unwrap();

        assert!(graph.validate().is_err());
    }

    #[test]
    fn test_parallel_levels() {
        let mut graph = TaskGraph::new();
        graph
            .add_task(make_task("deps", &[], TaskAction::Internal(InternalTask::ResolveDeps)))
            .unwrap();
        graph
            .add_task(make_task("compile", &["deps"], TaskAction::Internal(InternalTask::Compile)))
            .unwrap();
        graph
            .add_task(make_task("test", &["compile"], TaskAction::Internal(InternalTask::Test)))
            .unwrap();
        graph
            .add_task(make_task("lint", &["compile"], TaskAction::Composite))
            .unwrap();

        let levels = graph.parallel_levels().unwrap();

        // Nivel 0: deps (sin dependencias)
        assert!(levels[0].contains(&"deps".to_string()));
        // Nivel 1: compile (depende de deps)
        assert!(levels[1].contains(&"compile".to_string()));
        // Nivel 2: test y lint (ambos dependen de compile, PARALELOS)
        assert!(levels[2].contains(&"test".to_string()));
        assert!(levels[2].contains(&"lint".to_string()));
    }
}
