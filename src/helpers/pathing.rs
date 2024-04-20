use eyre::Result;

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum Operation {
  Sub,
  Add,
  Mul
}

#[derive(Debug, Default, Clone)]
struct Node {
  data:u16,
  ///References (the indices) to all the [`Node`]s connected to this node.
  neighbors:Vec<(Operation, usize)>
}

pub struct Graph {
  nodes:[Node; 8]
}

impl Graph {
  pub fn new() -> Self {
    Graph {
      nodes:[
        Node::default(),
        Node::default(),
        Node::default(),
        Node::default(),
        Node::default(),
        Node::default(),
        Node::default(),
        Node::default()
      ]
    }
  }

  pub fn add_node(&mut self, index:usize, data:u16, neighbors:&'static [(Operation, usize)]) {
    let node = Node {
      data,
      neighbors:Vec::from(neighbors)
    };
    self.nodes[index] = node;

    //Add duplicates to neighbors
    for n in neighbors {
      self.nodes[n.1].neighbors.push((n.0, index));
    }
  }

  pub fn get_path(&self) {
    //Create the queue of nodes to process
    // let mut queue = Vec::new();

    //Create the goal, a tuple of the cell and score
    let goal = (7, 30);

    let mut current = (0, 22);

    for node in &self.nodes {
      //Update the current (cell, score) value
      if current == goal {
        break;
      }
    }
  }

  // ///Calculates the shortest path between the start node and the end node.
  // pub fn get_path(&self) {
  //   let mut visited = Vec::new();
  //   let mut queue = self.cells.clone();

  //   for i in 0..self.cells.len() {
  //     //Get the current node
  //     let current = self.cells[i];
  //     visited.push(current);

  //     //Get the node's neighbors
  //     let neighbors = self.get_neighbors(i);
  //   }
  // }

  // fn get_neighbors(&self, index:usize) -> Vec<u16> {
  //   //Indices of the cells to the right, above, left, below
  //   let indices = [index + 1, index + 4, index - 1, index - 4];
  //   let mut neighbors = Vec::new();
  //   for i in indices {
  //     if i < self.cells.len() {
  //       neighbors.push(self.cells[i]);
  //     }
  //   }
  //   neighbors
  // }
}
