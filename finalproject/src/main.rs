use petgraph::dot::{Config, Dot};
use petgraph::graph::DiGraph;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead};
use std::io::Write;
use k_means::KMeans;
use plotters::prelude::*;
use plotters::style::PointStyle;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EducationData {
    municipality: String,
    data: HashMap<i32, (i32, f64)>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PopGrowthData {
    municipality: String,
    data: HashMap<i32, (i32, f64)>,
}

trait GraphData {
    fn municipality(&self) -> &str;
    fn get_weight(&self, category: i32) -> Option<f64>;
}

impl GraphData for EducationData {
    fn municipality(&self) -> &str {
        &self.municipality
    }

    fn get_weight(&self, category: i32) -> Option<f64> {
        self.data.get(&category).map(|&(_, weight)| weight)
    }
}

impl GraphData for PopGrowthData {
    fn municipality(&self) -> &str {
        &self.municipality
    }

    fn get_weight(&self, category: i32) -> Option<f64> {
        self.data.get(&category).map(|&(_, weight)| weight)
    }
}

fn visualize_graph<T>(
    graph: &DiGraph<&str, T>,
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Serialize + std::fmt::Debug,
{
    let mut file = File::create(file_path)?;
    let dot = Dot::with_config(graph, &[Config::EdgeNoLabel]);
    write!(file, "{:?}", dot)?;

    Ok(())
}

fn main() {
    // Read education data
    let education_data =
        read_data::<EducationData>("educationstats.txt").expect("Error reading education data");

    // Read pop growth data
    let pop_growth_data =
        read_data::<PopGrowthData>("popgrowthstats.txt").expect("Error reading pop growth data");

    // Filter common municipalities
    let common_municipalities = filter_common_municipalities(&education_data, &pop_growth_data);

    // Create graphs from your data
    let education_graph: DiGraph<&str, f64> = create_graph(&education_data);
    let pop_growth_graph: DiGraph<&str, f64> = create_graph(&pop_growth_data);

    // Visualize the graphs
    if let Err(err) = visualize_graph(&education_graph, "education_graph.dot") {
        eprintln!("Error: {}", err);
    }

    if let Err(err) = visualize_graph(&pop_growth_graph, "pop_growth_graph.dot") {
        eprintln!("Error: {}", err);
    }

    // Perform k-means clustering
    let k = 3; // Specify the number of clusters
    let education_clusters = k_means_clustering(&education_data, k);
    let pop_growth_clusters = k_means_clustering(&pop_growth_data, k);

    // Output clusters
    println!("Education Clusters: {:?}", education_clusters);
    println!("Pop Growth Clusters: {:?}", pop_growth_clusters);

    // Plot clusters (example using plotters crate)
    plot_clusters(&education_clusters, "education_clusters.png");
    plot_clusters(&pop_growth_clusters, "pop_growth_clusters.png");
}

fn read_data<T: for<'de> Deserialize<'de>>(
    file_path: &str,
) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let records: Result<Vec<T>, _> = serde_json::from_reader(reader);
    records
}

fn create_graph<T: GraphData>(data: &[T]) -> DiGraph<&str, f64> {
    // Your existing implementation
}

fn k_means_clustering<T: GraphData>(data: &[T], k: usize) -> HashMap<usize, Vec<&str>> {
    // Extract features for k-means clustering
    let features: Vec<Vec<f64>> = data
        .iter()
        .map(|entry| {
            (1..=10)
                .map(|category| entry.get_weight(category).unwrap_or(0.0))
                .collect()
        })
        .collect();

    // Perform k-means clustering
    let kmeans = KMeans::new(k);
    let clusters = kmeans.fit(&features);

    // Organize results into HashMap
    let mut result_clusters: HashMap<usize, Vec<&str>> = HashMap::new();
    for (i, cluster_idx) in clusters.iter().enumerate() {
        result_clusters
            .entry(*cluster_idx)
            .or_insert_with(Vec::new)
            .push(data[i].municipality());
    }

    result_clusters
}

fn filter_common_municipalities<T>(
    data1: &[T],
    data2: &[T],
) -> Vec<(String, T, T)>
where
    T: Clone + PartialEq,
{
    let mut common_municipalities = Vec::new();

    for entry1 in data1.iter() {
        if let Some(entry2) = data2.iter().find(|&e| e == entry1) {
            common_municipalities.push((
                entry1.municipality().to_string(),
                entry1.clone(),
                entry2.clone(),
            ));
        }
    }

    common_municipalities
}

// Function to plot clusters using plotters crate
fn plot_clusters(clusters: &HashMap<usize, Vec<&str>>, file_path: &str) {
    let root = BitMapBackend::new(file_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("Cluster Plot", ("sans-serif", 40).into_font())
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_ranged(0f64..10f64, 0f64..10f64)
        .unwrap();

    for (_, points) in clusters {
        let mut x_points = Vec::new();
        let mut y_points = Vec::new();

        for municipality in points {
            if let Some((x, y)) = find_coordinates_for_municipality(municipality) {
                x_points.push(x);
                y_points.push(y);
            }
        }

        chart
            .draw_series(
                Points::of_element(
                    x_points.iter().zip(y_points.iter()),
                    5,
                    &BLACK,
                    &|c, s, st| {
                        return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                            +
                            &Circle::new((2, 2), 5, BLUE.filled()), // Choose the marker style
                            &|_| {},
                        )
                )
                .unwrap();
    }
}

// Function to find coordinates for a municipality (example implementation)
fn find_coordinates_for_municipality(municipality: &str) -> Option<(f64, f64)> {
    // Replace this with a real implementation based on your data
    // For example, you might have latitude and longitude data for municipalities
    // Here, I'm just returning a random point for illustration purposes
    let mut rng = rand::thread_rng();
    Some((rng.gen_range(0.0..10.0), rng.gen_range(0.0..10.0)))
}

