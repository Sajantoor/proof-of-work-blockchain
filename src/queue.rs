use std::sync::mpsc;
use std::thread;

pub trait Task {
    type Output: Send;
    fn run(&self) -> Option<Self::Output>;
}

pub struct WorkQueue<TaskType: 'static + Task + Send> {
    send_tasks: Option<spmc::Sender<TaskType>>, // Option because it will be set to None to close the queue
    recv_tasks: spmc::Receiver<TaskType>,
    recv_output: mpsc::Receiver<TaskType::Output>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl<TaskType: 'static + Task + Send> WorkQueue<TaskType> {
    // Create the channels; start the worker threads; record their JoinHandles
    pub fn new(n_workers: usize) -> WorkQueue<TaskType> {
        let (send_tasks, recv_tasks) = spmc::channel();
        let (send_output, recv_output) = mpsc::channel();

        let mut workers = Vec::new();
        // create threads to be workers and add their handles to the vec
        for _ in 0..n_workers {
            let recv_clone = recv_tasks.clone();
            let snd_clone = send_output.clone();
            let worker = thread::spawn(|| Self::run(recv_clone, snd_clone));
            workers.push(worker);
        }

        return WorkQueue {
            send_tasks: Some(send_tasks),
            recv_tasks,
            recv_output,
            workers,
        };
    }

    // The main logic for a worker thread
    fn run(recv_tasks: spmc::Receiver<TaskType>, send_output: mpsc::Sender<TaskType::Output>) {
        loop {
            let task_recv = recv_tasks.recv();

            // NOTE: task_result will be Err() if the spmc::Sender has been destroyed and no more messages can be received here
            if let Result::Err(_) = task_recv {
                return;
            }

            let task = task_recv.unwrap();
            let task_result = task.run();

            // if tasks result was None, do nothing
            // if task result is Some, send the result to the main thread
            if let Option::Some(task_value) = task_result {
                let send_result = send_output.send(task_value);
                // handle error when sending
                if let Result::Err(_) = send_result {
                    break;
                }
            }
        }
    }

    // Send this task to a worker
    pub fn enqueue(&mut self, t: TaskType) -> Result<(), spmc::SendError<TaskType>> {
        match &mut self.send_tasks {
            Some(send_tasks) => return send_tasks.send(t),
            // If the send_tasks field is None, we can panic.
            None => panic!("Sender is None, when enqueuing."),
        }
    }

    // Helper methods that let you receive results in various ways
    pub fn iter(&mut self) -> mpsc::Iter<TaskType::Output> {
        self.recv_output.iter()
    }
    pub fn recv(&mut self) -> TaskType::Output {
        self.recv_output
            .recv()
            .expect("I have been shutdown incorrectly")
    }
    pub fn try_recv(&mut self) -> Result<TaskType::Output, mpsc::TryRecvError> {
        self.recv_output.try_recv()
    }
    pub fn recv_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> Result<TaskType::Output, mpsc::RecvTimeoutError> {
        self.recv_output.recv_timeout(timeout)
    }

    pub fn shutdown(&mut self) {
        // Destroy the spmc::Sender so everybody knows no more tasks are incoming;
        // drain any pending tasks in the queue; wait for each worker thread to finish.
        // HINT: Vec.drain(..)

        // Destroy the sender
        self.send_tasks = None;

        // Drain any pending tasks in the queue
        // TODO: This is not optimal
        let mut tasks = self.recv_tasks.recv();
        while let Result::Ok(_) = tasks {
            tasks = self.recv_tasks.recv();
        }

        // TODO: This isn't optimal either
        let remaining_workers = self.workers.drain(0..);
        for worker in remaining_workers {
            worker.join().unwrap();
        }
    }
}

impl<TaskType: 'static + Task + Send> Drop for WorkQueue<TaskType> {
    fn drop(&mut self) {
        // "Finalisation in destructors" pattern: https://rust-unofficial.github.io/patterns/idioms/dtor-finally.html
        match self.send_tasks {
            None => {} // already shut down
            Some(_) => self.shutdown(),
        }
    }
}
