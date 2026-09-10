/**
 * Simple task queuing system which ensures that each method runs sequentially
 * in the order of execution
 */
export class TaskQueue {
  private taskList: Promise<unknown> = Promise.resolve()

  public queue<T>(task: () => Promise<T>): Promise<T> {
    const result = this.taskList.then(() => task())

    // Catches are left on the internal promise tail so that even if a promise
    // rejects, it does not stall future tasks
    this.taskList = result.catch((err) => {
      console.error(`TaskQueue error: ${err}`)
    })

    return result
  }
}
