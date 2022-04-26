import React, { useState, useEffect } from 'react';
import PropTypes from 'prop-types';

const Marketplace = ({currentUser, contract}) => {
    const [nfts, setNfts] = useState([]);
  
  useEffect(() => {
      async function fetchData() {
          const count = await contract.nft_total_supply();
          const result = await contract.nft_tokens(
          {
              from_index: "0",
              limit: parseInt(count)
          });
          console.log(result);
          setNfts(splitArrayIntoChunksOfLen(result, 3));
      }
      
      fetchData();
  }, [contract, currentUser]);
  
  const splitArrayIntoChunksOfLen = (arr, len) => {
    var chunks = [], i = 0, n = arr.length;
    while (i < n) {
      chunks.push(arr.slice(i, i += len));
    }
    return chunks;
  }
  
   return <>
                 <header>
                   <h1>All NFTs that where minted.</h1>
                 </header>
                 
                  {nfts.map(chunk => 
                  <div className="row">
                    {chunk.map(nft =>
                      <div className="col s4">
                          <div className="card">
                            <div className="card-image">
                              <img src={nft.metadata.media} alt={nft.metadata.title}/>
                            </div>
                            <div className="card-title">{nft.metadata.title}</div>
                            <div className="card-content">
                              <p>{nft.metadata.description}</p>
                            </div>
                          </div>
                      </div>)}          
                  </div>)}
          </>
}

Marketplace.propTypes = {
  currentUser: PropTypes.shape({
    accountId: PropTypes.string.isRequired,
    balance: PropTypes.string.isRequired
  }),
  contract: PropTypes.shape({
    nft_total_supply: PropTypes.func.isRequired,
    nft_tokens: PropTypes.func.isRequired
  }).isRequired,
};

export default Marketplace;