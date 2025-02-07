use bellman::groth16::{Parameters, VerifyingKey};
use bls12_381::{Bls12, G1Affine, G2Affine};
use byteorder::{BigEndian, ReadBytesExt};
use group::UncompressedEncoding;
use rayon::prelude::*;
use std::io::{self, Read};
use std::sync::Arc;

pub fn read<R: Read>(mut reader: R, checked: bool) -> io::Result<Parameters<Bls12>> {
    use std::time::Instant;
    let now = Instant::now();
    let read_g1 = |reader: &mut R| -> io::Result<<G1Affine as UncompressedEncoding>::Uncompressed> {
        let mut repr = <G1Affine as UncompressedEncoding>::Uncompressed::default();
        reader.read_exact(repr.as_mut())?;
        Ok(repr)
    };
    let process_g1 =
        |repr: &<G1Affine as UncompressedEncoding>::Uncompressed| -> io::Result<G1Affine> {
            let affine = if checked {
                <G1Affine as UncompressedEncoding>::from_uncompressed(repr)
            } else {
                <G1Affine as UncompressedEncoding>::from_uncompressed_unchecked(repr)
            };

            let affine = if affine.is_some().into() {
                Ok(affine.unwrap())
            } else {
                Err(io::Error::new(io::ErrorKind::InvalidData, "invalid G1"))
            };

            affine.and_then(|e| {
                if e.is_identity().into() {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "point at infinity",
                    ))
                } else {
                    Ok(e)
                }
            })
        };

    let read_g2 = |reader: &mut R| -> io::Result<<G2Affine as UncompressedEncoding>::Uncompressed> {
        let mut repr = <G2Affine as UncompressedEncoding>::Uncompressed::default();
        reader.read_exact(repr.as_mut())?;
        Ok(repr)
    };

    let process_g2 =
        |repr: &<G2Affine as UncompressedEncoding>::Uncompressed| -> io::Result<G2Affine> {
            let affine = if checked {
                <G2Affine as UncompressedEncoding>::from_uncompressed(repr)
            } else {
                <G2Affine as UncompressedEncoding>::from_uncompressed_unchecked(repr)
            };

            let affine = if affine.is_some().into() {
                Ok(affine.unwrap())
            } else {
                Err(io::Error::new(io::ErrorKind::InvalidData, "invalid G2"))
            };

            affine.and_then(|e| {
                if e.is_identity().into() {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "point at infinity",
                    ))
                } else {
                    Ok(e)
                }
            })
        };

    let vk = VerifyingKey::<Bls12>::read(&mut reader)?;

    let h;
    let l;
    let a;
    let b_g1;
    let b_g2;

    {
        let len = reader.read_u32::<BigEndian>()? as usize;
        let mut h_raw = Vec::with_capacity(len);
        for _ in 0..len {
            h_raw.push(read_g1(&mut reader)?);
        }
        h = h_raw
            .par_iter()
            .map(process_g1)
            .collect::<io::Result<Vec<_>>>()?;
    }

    {
        let len = reader.read_u32::<BigEndian>()? as usize;
        let mut l_raw = Vec::with_capacity(len);
        for _ in 0..len {
            l_raw.push(read_g1(&mut reader)?);
        }
        l = l_raw
            .par_iter()
            .map(process_g1)
            .collect::<io::Result<Vec<_>>>()?;
    }

    {
        let len = reader.read_u32::<BigEndian>()? as usize;
        let mut a_raw = Vec::with_capacity(len);
        for _ in 0..len {
            a_raw.push(read_g1(&mut reader)?);
        }
        a = a_raw
            .par_iter()
            .map(process_g1)
            .collect::<io::Result<Vec<_>>>()?;
    }

    {
        let len = reader.read_u32::<BigEndian>()? as usize;
        let mut b_g1_raw = Vec::with_capacity(len);
        for _ in 0..len {
            b_g1_raw.push(read_g1(&mut reader)?);
        }
        b_g1 = b_g1_raw
            .par_iter()
            .map(process_g1)
            .collect::<io::Result<Vec<_>>>()?;
    }

    {
        let len = reader.read_u32::<BigEndian>()? as usize;
        let mut b_g2_raw = Vec::with_capacity(len);
        for _ in 0..len {
            b_g2_raw.push(read_g2(&mut reader)?);
        }
        b_g2 = b_g2_raw
            .par_iter()
            .map(process_g2)
            .collect::<io::Result<Vec<_>>>()?;
    }

    Ok(Parameters {
        vk,
        h: Arc::new(h),
        l: Arc::new(l),
        a: Arc::new(a),
        b_g1: Arc::new(b_g1),
        b_g2: Arc::new(b_g2),
    })
}
