use std::net::IpAddr;
use trust_dns_resolver::TokioAsyncResolver;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::sync::{Mutex, Arc};
use futures::stream::StreamExt;
use tokio::time::Duration;

async fn resolve_dns_a(domain: &str) -> Result<Vec<IpAddr>, Box<dyn std::error::Error>> {
    let resolver = TokioAsyncResolver::tokio_from_system_conf()?;
    let response = resolver.lookup_ip(domain).await?;
    let ip_addrs: Vec<IpAddr> = response.iter().collect();
    println!("{:?}",response);
    Ok(ip_addrs)
}

async fn dns_req(domain: &str) -> bool {
    match resolve_dns_a(domain).await {
        Ok(_) => true,
        Err(_) => false,
    }
}

async fn parse_file(filename: &str) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    for line in reader.lines() {
        let line = line?;
        lines.push(line);
    }

    Ok(lines)
}

pub async fn enuml(filename: &str, domain: &str) -> Vec<String> {
    let sub_domains = Arc::new(Mutex::new(Vec::<String>::new()));
    let cloned_domain = Arc::new(domain.to_string());
    match parse_file(filename).await {
        Ok(payloads) => {
            let tasks = futures::stream::iter(payloads.into_iter()).map(|subd| {
                let sub_domains = Arc::clone(&sub_domains);
                println!("{:?}",&format!("{}.{}", &subd, &cloned_domain));
                let cloned_domain = Arc::clone(&cloned_domain);
                tokio::spawn(async move {
                    // Sleep to avoid overwhelming the DNS resolver
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    let res = resolve_dns_a(&format!("{}.{}", &subd, &cloned_domain)).await;
                    println!("{:?}",res);
                    match res{
                      Ok(value)=>{sub_domains.lock().unwrap().push(format!("{}.{}",subd.to_string(),cloned_domain));}
                      Err(err) => {}
                    
                    }})
                    
                
            });

            let tasks = tasks
                .buffer_unordered(40)
                .collect::<Vec<_>>()
                .await;

            for result in tasks {
              if let Ok(open_port) = result {
                 println!("{:?}", open_port);
              }
             }    

            Arc::try_unwrap(sub_domains)
                .expect("Failed to unwrap Arc")
                .into_inner()
                .expect("Failed to get inner data from Mutex")
        }
        Err(err) => {
            eprintln!("Error parsing file: {:?}", err);
            Vec::new()
        }
    }
}