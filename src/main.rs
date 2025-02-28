// main.rs
#[tokio::main]
async fn main() {
    println!("Hello, world!");

    // 初始化
    let init_task = tokio::spawn({
        async move {
            new_pbft::init().await.unwrap();
        }
    });


    // 等待所有任务执行完毕
    tokio::try_join!(init_task).unwrap();
}
