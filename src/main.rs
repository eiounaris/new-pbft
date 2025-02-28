#![allow(dead_code, unused_variables)]


// main.rs
#[tokio::main]
async fn main() -> Result<(), String> {
    // 初始化
    match new_pbft::init().await {
        Ok((
            system_config, 
            client, 
            state, 
            pbft, 
            reset_sender, 
            reset_receiver,
        )) => {

            // 视图请求
            tokio::spawn({
                let system_config = system_config.clone();
                let client = client.clone();
                let pbft = pbft.clone();
                async move {
                    if let Err(e) = new_pbft::view_request(system_config, client, pbft).await {
                        eprintln!("{e:?}");
                    }
                }
            });

            // 消息处理
            let recv_task = tokio::spawn({
                let system_config = system_config.clone();
                let client = client.clone();
                let state = state.clone();
                let pbft = pbft.clone();
                let reset_sender = reset_sender.clone();
                async move {
                    if let Err(e) = new_pbft::handle_message(system_config, client, state, pbft, reset_sender).await {
                        eprintln!("\n{e:?}");
                    }
                }
            });

            // 主节点心跳
            let heartbeat_task = tokio::spawn({
                let system_config = system_config.clone();
                let client = client.clone();
                let pbft = pbft.clone();
                async move {
                    if let Err(e) = new_pbft::heartbeat(system_config, client, pbft).await {
                        eprintln!("\n{e:?}");
                    }
                }
            });

            // 从节点视图切换
            let view_change_task = tokio::spawn({
                let system_config = system_config.clone();
                let client = client.clone();
                let pbft = pbft.clone();
                async move {
                    if let Err(e) = new_pbft::view_change(system_config, client, pbft, reset_receiver).await {
                        eprintln!("\n{e:?}");
                    }
                }
            });

            // 命令行输入
            let send_task = tokio::spawn({
                let client = client.clone();
                let system_config =  system_config.clone();
                async move {
                    if let Err(e) = new_pbft::send_message(client, system_config).await {
                        eprintln!("\n{e:?}");
                    }
                }
            });


            // 等待所有任务执行完毕
            tokio::try_join!(recv_task, heartbeat_task, view_change_task, send_task).unwrap();
        },
        
        Err(e) => eprintln!("\n{e:?}"),
    }
    
    Ok(())
}
