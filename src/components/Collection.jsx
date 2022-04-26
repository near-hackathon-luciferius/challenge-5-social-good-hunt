import React, { useState, useEffect } from 'react';
import PropTypes from 'prop-types';
import Form from './Form';

const Collection = ({currentUser, onNftMint, errorMessage, contract}) => {
    const [nfts, setNfts] = useState([]);
  
  useEffect(() => {
      async function fetchData() {
        const count = await contract.nft_supply_for_owner({account_id: currentUser.accountId});
        const result = await contract.nft_tokens_for_owner(
        {
            account_id: currentUser.accountId,
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
                   <h1>{currentUser.accountId}'s Collection</h1>
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
                 <h5>Mint a new NFT below.</h5>
                 <Form onNftMint={onNftMint} errorMessage={errorMessage} />
          </>
}

Collection.propTypes = {
  onNftMint: PropTypes.func.isRequired,
  errorMessage: PropTypes.string.isRequired,
  currentUser: PropTypes.shape({
    accountId: PropTypes.string.isRequired,
    balance: PropTypes.string.isRequired
  })
};

export default Collection;