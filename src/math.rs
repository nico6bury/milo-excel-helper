
pub fn avg(data: &Vec<f32>) -> f32 {
	let sum = data.iter().sum::<f32>() as f32;
	let count = data.len() as f32;
	return sum / count;
}//end avg()

pub fn std(data: &Vec<f32>) -> f32 {
	let data_mean = avg(data);
	let count = data.len();
	let variance = data.iter().map(|value| {
		let diff = data_mean - (*value as f32);
		diff * diff
	}).sum::<f32>() / count as f32;
	return variance.sqrt();
}//end std()

pub fn cv(data: &Vec<f32>) -> f32 {
	return std(data) / avg(data);
}//end cv()