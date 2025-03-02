#![allow(dead_code, unused_variables)]


// main.rs
#[tokio::main]
async fn main() -> Result<(), String> {
    // 初始化
    match new_pbft::init().await {
        Ok((
            constant_config, 
            variable_config,
            client, 
            state, 
            pbft, 
            reset_sender, 
            reset_receiver,
        )) => {

            // 视图请求
            tokio::spawn({
                let constant_config = constant_config.clone();
                let client = client.clone();
                let pbft = pbft.clone();
                async move {
                    if let Err(e) = new_pbft::view_request(constant_config, client, pbft).await {
                        eprintln!("{e:?}");
                    }
                }
            });

            // 消息处理
            let recv_task = tokio::spawn({
                let constant_config = constant_config.clone();
                let variable_config = variable_config.clone();
                let client = client.clone();
                let state = state.clone();
                let pbft = pbft.clone();
                let reset_sender = reset_sender.clone();
                async move {
                    if let Err(e) = new_pbft::handle_message(constant_config, variable_config, client, state, pbft, reset_sender).await {
                        eprintln!("{e:?}");
                    }
                }
            });

            // 主节点心跳
            let heartbeat_task = tokio::spawn({
                let constant_config = constant_config.clone();
                let variable_config = variable_config.clone();
                let client = client.clone();
                let pbft = pbft.clone();
                async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await; // 硬编码，等待状态稳定
                    if let Err(e) = new_pbft::heartbeat(constant_config, variable_config, client, pbft).await {
                        eprintln!("{e:?}");
                    }
                }
            });

            // 从节点视图切换
            let view_change_task = tokio::spawn({
                let constant_config = constant_config.clone();
                let variable_config = variable_config.clone();
                let client = client.clone();
                let pbft = pbft.clone();
                async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await; // 硬编码，等待状态稳定
                    if let Err(e) = new_pbft::view_change(constant_config, variable_config, client, pbft, reset_receiver).await {
                        eprintln!("{e:?}");
                    }
                }
            });

            // 命令行输入
            let send_task = tokio::spawn({
                let constant_config = constant_config.clone();
                let variable_config = variable_config.clone();
                let client = client.clone();
                let state = state.clone();
                async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await; // 硬编码，等待状态稳定
                    if let Err(e) = new_pbft::send_message(constant_config, client, state).await {
                        eprintln!("{e:?}");
                    }
                }
            });


            // 等待所有任务执行完毕
            tokio::try_join!(recv_task, heartbeat_task, view_change_task, send_task).unwrap();
        },
        
        Err(e) => eprintln!("{e:?}"),
    }
    
    Ok(())
}
