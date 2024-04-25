use crate::vm::WORDSIZE;
use std::{
  collections::{HashSet, VecDeque},
  fs,
  io::Write
};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Operation {
  #[default]
  Sub,
  Add,
  Mul
}

//Nodes are better when no edges
#[derive(Debug, Default, Clone)]
struct Node {
  data:u16,
  ///References (the indices) to all the [`Node`]s connected to this node.
  neighbors:Vec<Edge>
}

#[derive(Debug, Default, Clone)]
struct Edge {
  op:Operation,
  next:usize
}

impl Edge {
  fn new(op:Operation, next:usize) -> Self {
    Edge { op, next }
  }
}

pub struct Graph {
  nodes:VecDeque<Node> // adj_list:Vec<Vertex>
}

impl Graph {
  pub fn new(cap:usize) -> Self {
    let mut nodes = VecDeque::new();
    nodes.resize(cap, Node::default());
    Graph { nodes }
  }

  pub fn add_node(&mut self, cell:usize, data:u16, neighbors:Vec<(Operation, usize)>) {
    let mut edges = Vec::new();
    for (op, cell) in neighbors {
      edges.push(Edge::new(op, cell));
    }

    let node = Node { data, neighbors:edges };
    self.nodes[cell] = node;
  }

  ///Goal params take in tuples of (number, tile).
  pub fn get_shortest_path(&mut self, start:(usize, u16), goal:(usize, u16)) {
    //Create the file where the path will print
    let mut file = fs::File::create("path.txt").unwrap();

    //Push the start (cell, value) to the queue
    let start = (start.0, start.1, vec![]);
    let mut queue = VecDeque::new();
    queue.push_back(start);

    //Initialize the the HashSet of visited cells
    let mut visited = HashSet::new();

    'path: while !queue.is_empty() {
      let (current_cell, current_val, path) = queue.pop_front().unwrap();

      //Break if a path has been found
      if (current_cell, current_val) == goal {
        //Return the path
        write!(file, "{:?}", path).unwrap();
        break 'path;
      }

      //Explore all the current node's neighbors
      for edge in &self.nodes[current_cell].neighbors {
        //Check if the neighbor was visited
        if !visited.contains(&(current_val, edge.next)) {
          //Get the value of the path at the neighbor
          let current_val = self.calc_value(current_val, edge);

          //Add the neighbor to the queue
          let mut new_path = path.clone();
          let path_seg = format!("{current_cell} + {:?}", edge.op);
          new_path.push(path_seg);

          //New state of the path (cell, value, previous)
          let new_path_state = (edge.next, current_val, new_path);
          queue.push_back(new_path_state.clone());

          //Update visited
          visited.insert((current_val, edge.next));
        }
      }
    }
  }

  ///Given the values of a start cell, a destination cell, and the edge between
  /// them, returns the value at the destination cell.
  fn calc_value(&self, start_val:u16, edge:&Edge) -> u16 {
    let current_val;
    match edge.op {
      Operation::Add => current_val = start_val + self.nodes[edge.next].data,
      Operation::Sub =>
      //  current_val = start_val - self.nodes[edge.next].data,
      {
        if start_val < self.nodes[edge.next].data {
          current_val = 0
        }
        else {
          current_val = start_val - self.nodes[edge.next].data
        }
      }
      Operation::Mul => current_val = (start_val as u32 * self.nodes[edge.next].data as u32) as u16 % WORDSIZE
    }
    current_val
  }
}

#[cfg(test)]
mod test {
  use super::{
    Graph,
    Operation::{Add, Mul, Sub}
  };

  #[test]
  fn path() {
    //Create the graph
    let mut graph = Graph::new(8);
    graph.add_node(0, 8, vec![(Sub, 1), (Sub, 3), (Mul, 3), (Mul, 4), (Mul, 2)]);
    graph.add_node(1, 1, vec![(Mul, 5), (Mul, 3), (Sub, 0), (Sub, 3)]);
    graph.add_node(2, 4, vec![(Mul, 0), (Mul, 3), (Mul, 4), (Add, 4)]);
    graph.add_node(3, 11, vec![(Sub, 0), (Sub, 1), (Mul, 0), (Mul, 1), (Mul, 4), (Mul, 5), (Sub, 7), (Mul, 2), (Sub, 4), (Sub, 5)]);
    graph.add_node(4, 4, vec![(Mul, 2), (Mul, 0), (Mul, 3), (Add, 2), (Sub, 3), (Sub, 5), (Sub, 7)]);
    graph.add_node(5, 18, vec![(Sub, 7), (Sub, 4), (Sub, 3), (Mul, 3), (Mul, 1), (Mul, 7)]);
    graph.add_node(6, 22, vec![(Sub, 7), (Sub, 4), (Add, 4), (Add, 2)]);
    graph.add_node(7, 9, vec![(Sub, 4), (Sub, 3), (Sub, 5), (Mul, 5)]);

    //Calculate the path
    graph.get_shortest_path((6, 22), (1, 30));
  }
}
